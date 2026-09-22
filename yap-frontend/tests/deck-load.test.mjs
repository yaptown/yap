import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import vm from "node:vm";
import ts from "typescript";
import * as reducers from "../../yap-frontend-rs/pkg/yap_frontend_rs_bg.js";

// Actual Rust decisions; build the web package before running this test.
const { instance } = await WebAssembly.instantiate(
  readFileSync(
    new URL(
      "../../yap-frontend-rs/pkg/yap_frontend_rs_bg.wasm",
      import.meta.url,
    ),
  ),
  { "./yap_frontend_rs_bg.js": reducers },
);
reducers.__wbg_set_wasm(instance.exports);
instance.exports.__wbindgen_start();
const source = ts.transpileModule(
  readFileSync(new URL("../src/core/useDeck.ts", import.meta.url), "utf8"),
  {
    compilerOptions: {
      module: ts.ModuleKind.CommonJS,
      target: ts.ScriptTarget.ES2022,
    },
  },
).outputText;

function deferred() {
  let resolve, reject;
  const promise = new Promise((yes, no) => {
    resolve = yes;
    reject = no;
  });
  return { promise, resolve, reject };
}
function harness({ ready = true, cached = false } = {}) {
  const slots = [],
    packs = [],
    builds = [],
    errors = [];
  const storage = new Map();
  let selection = { nativeLanguage: "English", targetLanguage: "French" };
  if (cached) storage.set("yap-last-course", JSON.stringify(selection));
  let inputs = "none",
    cursor = 0,
    dirty = false,
    pending = [],
    result;
  const same = (a, b) =>
    a?.length === b?.length && a.every((v, i) => Object.is(v, b[i]));
  const react = {
    useState(initial) {
      const i = cursor++;
      slots[i] ??= {
        value: typeof initial === "function" ? initial() : initial,
      };
      return [
        slots[i].value,
        (value) => {
          slots[i].value =
            typeof value === "function" ? value(slots[i].value) : value;
          dirty = true;
        },
      ];
    },
    useRef(initial) {
      return react.useState(() => ({ current: initial }))[0];
    },
    useMemo(fn, deps) {
      const i = cursor++;
      if (!same(slots[i]?.deps, deps)) slots[i] = { value: fn(), deps };
      return slots[i].value;
    },
    useCallback(fn, deps) {
      return react.useMemo(() => fn, deps);
    },
    useEffect(effect, deps) {
      const i = cursor++;
      if (!same(slots[i]?.deps, deps))
        pending.push(() => {
          slots[i]?.cleanup?.();
          slots[i] = { deps, effect, cleanup: effect() };
        });
    },
    useLayoutEffect(effect, deps) {
      react.useEffect(effect, deps);
    },
    useSyncExternalStore(_subscribe, getSnapshot) {
      return getSnapshot();
    },
  };
  const pack = (stage, course, progress) => {
    const task = { ...deferred(), stage, course, progress };
    packs.push(task);
    return task.promise;
  };
  const weapon = {
    request_deck_selection() {},
    request_reviews() {},
    get_stream_num_events: () => (ready ? 0 : undefined),
    reviews_history_known: () => ready,
    get_deck_selection_state: () => selection,
    deck_inputs_key: () => inputs,
    load_language_pack_core: (course, progress) =>
      pack("core", course, progress),
    load_language_pack: (course, progress) => pack("full", course, progress),
    get_deck_state(course) {
      const task = { ...deferred(), course, inputs };
      builds.push(task);
      return task.promise;
    },
  };
  const exports = {};
  vm.runInNewContext(source, {
    exports,
    console,
    Date,
    Error,
    localStorage: {
      getItem: (k) => storage.get(k) ?? null,
      setItem: (k, v) => storage.set(k, v),
    },
    require(path) {
      if (path === "react") return react;
      if (path === "@/core/weapon") return { useWeapon: () => weapon };
      if (path === "@sentry/react")
        return {
          addBreadcrumb() {},
          captureException: (...args) => errors.push(args),
        };
      if (path.endsWith("/pkg")) return reducers;
      throw new Error(path);
    },
  });
  function render() {
    let attempts = 0;
    do {
      assert.ok(++attempts < 20, "render stabilizes");
      dirty = false;
      cursor = 0;
      result = exports.useDeck();
      const work = pending;
      pending = [];
      work.forEach((fn) => fn());
    } while (dirty);
    return result;
  }
  const flush = async () => {
    for (let i = 0; i < 5; i++) await Promise.resolve();
    return render();
  };
  return {
    packs,
    builds,
    errors,
    render,
    flush,
    setReady(value) {
      ready = value;
      return render();
    },
    setInputs(value) {
      inputs = value;
      return render();
    },
    select(targetLanguage) {
      selection = targetLanguage
        ? { nativeLanguage: "English", targetLanguage }
        : null;
      return render();
    },
    replay() {
      slots.forEach((s) => s?.cleanup?.());
      slots.forEach((s) => {
        if (s?.effect) s.cleanup = s.effect();
      });
      return render();
    },
    stop() {
      slots.forEach((s) => s?.cleanup?.());
    },
    async core() {
      inputs = "core";
      packs.at(-1).resolve();
      return flush();
    },
    async built(deck = {}) {
      builds.at(-1).resolve(deck);
      await flush();
      return deck;
    },
  };
}

test("core failure retries, but an upgrade failure/retry retains the usable snapshot", async () => {
  const h = harness();
  h.render();
  h.packs[0].reject(new Error("core failed"));
  let state = await h.flush();
  assert.equal(state.view.phase.type, "Error");
  state.retry();
  await h.flush();
  await h.core();
  const deck = await h.built();
  h.packs
    .at(-1)
    .reject({ detail: { type: "Download" }, toString: () => "offline" });
  state = await h.flush();
  assert.equal(state.view.phase.type, "Ready");
  assert.equal(state.deck, deck);
  assert.match(state.view.pack_banner.message, /offline/);
  state.retry();
  state = await h.flush();
  assert.equal(state.deck, deck);
  assert.equal(h.packs.at(-1).stage, "full");
  assert.equal(h.packs.filter((p) => p.stage === "core").length, 2);
  assert.equal(h.errors.length, 1);
});

test("cached core arriving before streams builds once streams become ready", async () => {
  const h = harness({ ready: false, cached: true });
  h.render();
  await h.core();
  assert.equal(h.builds.length, 0);
  h.setReady(true);
  assert.equal(h.builds.length, 1);
  await h.built(null);
  assert.equal(h.render().view.phase.type, "Loading");
  h.setInputs("full");
  h.packs.at(-1).resolve();
  await h.flush();
  await h.built();
  assert.equal(h.render().view.phase.type, "Ready");
});

test("StrictMode replay restarts initial load and ignores the cancelled lifetime", async () => {
  const h = harness();
  h.render();
  h.replay();
  assert.equal(h.packs.length, 2);
  h.packs[0].reject(new Error("stale"));
  await h.flush();
  assert.equal(h.errors.length, 0);
  await h.core();
  await h.built();
  assert.equal(h.render().view.phase.type, "Ready");
});

test("course changes and deselection reject stale progress, success, and failure", async () => {
  const h = harness();
  h.render();
  const stale = h.packs[0];
  h.select("Spanish");
  stale.progress("stale", 99);
  stale.resolve();
  await h.flush();
  assert.equal(h.builds.length, 0);
  const spanish = h.packs.at(-1);
  h.select(null);
  spanish.reject(new Error("stale Spanish"));
  const state = await h.flush();
  assert.equal(state.view.phase.type, "NoLanguageSelected");
  assert.equal(state.deck, null);
  assert.equal(h.errors.length, 0);
});

test("rebuilds do not blank the deck; unchanged and reverted inputs reject stale builds", async () => {
  const h = harness();
  h.render();
  await h.core();
  const deck = await h.built();
  h.setInputs("core");
  assert.equal(h.builds.length, 1);
  assert.equal(h.setInputs("B").deck, deck);
  const stale = h.builds.at(-1);
  h.setInputs("core");
  stale.resolve({ stale: true });
  await h.flush();
  assert.equal(h.render().deck, deck);
  h.setInputs("C");
  const afterUnmount = h.builds.at(-1);
  h.stop();
  afterUnmount.reject(new Error("stopped"));
  await h.flush();
  assert.equal(h.errors.length, 0);
});
