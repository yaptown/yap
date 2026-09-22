import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import vm from "node:vm";
import ts from "typescript";

function load(path, globals) {
  const source = ts.transpileModule(readFileSync(new URL(path, import.meta.url), "utf8"), {
    compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022, jsx: ts.JsxEmit.ReactJSX },
  }).outputText;
  const exports = {};
  vm.runInNewContext(source, { exports, ...globals });
  return exports;
}

function storageHarness() {
  const data = new Map();
  const localStorage = {
    getItem: (key) => data.get(key) ?? null,
    setItem: (key, value) => data.set(key, value),
    removeItem: (key) => data.delete(key),
  };
  const { PendingReview } = load("../src/review/challenges/pending-review.ts", { localStorage });
  return { data, localStorage, PendingReview };
}

test("draft slots isolate accounts/courses and discard mismatched or malformed snapshots", () => {
  const { data, PendingReview } = storageHarness();
  const slot = { key: "yap-pending-translation-one:French:English", identity: "v1-build-42-digest" };
  const decode = (state) => {
    if (!state?.phase) throw Error("invalid reducer state");
    return state;
  };
  const storage = new PendingReview(slot, decode);
  storage.save({ phase: "Graded" });
  assert.equal(storage.load().phase, "Graded");
  for (const scope of ["two:French:English", "one:Spanish:English"]) {
    assert.equal(new PendingReview({ ...slot, key: `yap-pending-translation-${scope}` }, decode).load(), undefined);
    assert.ok(data.has(slot.key));
  }
  const updated = new PendingReview({ ...slot, identity: "v1-build-43-digest" }, decode);
  assert.equal(updated.load(), undefined);
  assert.equal(data.has(slot.key), false);
  updated.save({ phase: "Editing" });
  assert.equal(updated.load().phase, "Editing", "discarding an obsolete draft still permits new saves");
  for (const raw of ["{bad json", JSON.stringify({ identity: slot.identity, payload: {} })]) {
    data.set(slot.key, raw);
    assert.equal(storage.load(), undefined);
    assert.equal(data.has(slot.key), false);
  }
});

test("storage failures never prevent review or completion", () => {
  const unavailable = () => { throw Error("storage blocked"); };
  const { PendingReview } = load("../src/review/challenges/pending-review.ts", {
    localStorage: { getItem: unavailable, setItem: unavailable, removeItem: unavailable },
  });
  const storage = new PendingReview({ key: "slot", identity: "identity" }, (state) => state);
  assert.equal(storage.load(), undefined);
  storage.save({ phase: "Graded" });
  storage.clear();
});

// Run the real challenge components' mount/resume and Complete handlers. Only
// bridge/network/rendering are stubbed; reducer transitions have Rust tests.
for (const kind of ["Translation", "Transcription"]) {
  for (const accepted of [false, true]) {
    test(`${kind} clears its completing draft only after acceptance (${accepted})`, () => {
      const { data, localStorage, PendingReview } = storageHarness();
      const effects = [];
      const slot = { key: `yap-pending-${kind.toLowerCase()}-one:French:English`, identity: "v1-build-42-digest" };
      const snapshot = { phase: { type: "Graded", completing: true, grade: { results: [] } }, inputs: [[0, "chat"]], text: "cat" };
      data.set(slot.key, JSON.stringify({ identity: slot.identity, payload: snapshot }));
      let completions = 0;
      let slotCalls = 0;
      const react = {
        useState: (initial) => [typeof initial === "function" ? initial() : initial, () => {}],
        useRef: (current) => ({ current }),
        useMemo: (fn) => fn(),
        useCallback: (fn) => fn,
        useEffect: (fn) => effects.push(fn),
        forwardRef: (fn) => fn,
      };
      const noop = () => {};
      const bridge = {
        get_app_version: () => "build",
        [`${kind.toLowerCase()}_pending_slot`]: (challenge, scope, version, count) => {
          slotCalls++;
          assert.equal(scope, "one:French:English");
          assert.equal(version, "build");
          assert.equal(count, 42n);
          assert.ok(challenge);
          return slot;
        },
        [`${kind.toLowerCase()}_resume`]: (state) => {
          if (kind === "Transcription") assert.ok(state.inputs instanceof Map);
          return { state, effects: [{ type: "Complete", outcome: { type: "Perfect" }, results: [], heteronyms_tapped: [], submission: "cat", completed_at_ms: 1234 }] };
        },
        [`${kind.toLowerCase()}_view`]: () => ({
          definitions: [], blanks: [], can_continue: false, words: [], proper_nouns: [],
          verdict: { perfect: true, word_grades: [], compare: [] },
        }),
        get_transcription_review_definitions: () => [],
      };
      const components = load(`../src/review/challenges/${kind}Challenge.tsx`, {
        Map, console, localStorage, window: { scrollTo: noop },
        require(path) {
          if (path === "react") return react;
          if (path === "react/jsx-runtime") return { jsx: noop, jsxs: noop };
          if (path.includes("yap-frontend-rs/pkg")) return bridge;
          if (path === "@/review/challenges/pending-review") return { PendingReview };
          if (path === "@/lib/movie-cache") return { getMovieMetadata: () => [] };
          if (path === "../../components/background-context") return { useBackground: () => ({ bumpBackground: noop }) };
          return new Proxy({}, { get: () => noop });
        },
      });
      components[`${kind}Challenge`]({
        sentence: { movie_titles: [] },
        challenge: { parts: [], movie_titles: [] },
        totalReviewsCompleted: 42n,
        pendingReviewScope: "one:French:English",
        onComplete: () => {
          completions++;
          if (completions > 1) {
            assert.equal(data.has(slot.key), !accepted, "resume replay must not resurrect an accepted draft");
            return false;
          }
          assert.ok(data.has(slot.key), "persist before requesting acceptance");
          assert.equal(JSON.parse(data.get(slot.key)).payload.phase.completing, true);
          return accepted;
        },
      });
      assert.equal(slotCalls, 1);
      // The first effect is resume; the others are focus/keyboard UI effects.
      effects[0]();
      assert.equal(completions, 1);
      assert.equal(data.has(slot.key), !accepted);
      effects[0](); // StrictMode repeats mount effects, but the controller rejects the second completion.
      assert.equal(completions, 2);
      assert.equal(data.has(slot.key), !accepted);
      if (!accepted && kind === "Transcription") {
        assert.deepEqual(JSON.parse(data.get(slot.key)).payload.inputs, [[0, "chat"]]);
      }
    });
  }
}

test("late no-op steps cannot resurrect an accepted draft", () => {
  const { data, PendingReview } = storageHarness();
  const slot = { key: "slot", identity: "identity" };
  const storage = new PendingReview(slot, (state) => state);
  storage.save({ completing: true });
  storage.clear();
  storage.save({ completing: true });
  assert.equal(data.has(slot.key), false);
});
