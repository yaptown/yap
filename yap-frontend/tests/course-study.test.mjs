import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import vm from "node:vm";
import ts from "typescript";

// Exercise the actual controller with deterministic hook slots and effects,
// following audio.test.mjs's transpile/VM pattern. No WASM/network/browser needed.
const source = ts.transpileModule(
  readFileSync(
    new URL("../src/contexts/course-study.tsx", import.meta.url),
    "utf8",
  ) + "\nexport { useStudyController };",
  {
    compilerOptions: {
      module: ts.ModuleKind.CommonJS,
      target: ts.ScriptTarget.ES2022,
      jsx: ts.JsxEmit.ReactJSX,
    },
  },
).outputText;

function harness() {
  const slots = [];
  const timers = new Map();
  const storage = new Map();
  const prefetches = [];
  const events = [];
  let now = 1_000_000;
  let audio = 0;
  let clips = 0;
  let online = true;
  let cursor = 0;
  let dirty = false;
  let pendingEffects = [];
  let poll;
  let restrictionReads = 0;
  let manifestRefreshes = 0;
  let selected;
  let due = Infinity;
  let totalReviews = 1n;
  let accomplishment = false;
  let placement = false;
  let session;
  let eventAvailable = true;
  const context = { userInfo: { id: "one", displayName: "Learner" } };
  const localStorage = {
    getItem: (key) => storage.get(key) ?? null,
    setItem: (key, value) => storage.set(key, value),
    removeItem: (key) => storage.delete(key),
  };
  const sameDeps = (a, b) =>
    a?.length === b?.length && a.every((value, i) => Object.is(value, b[i]));
  const react = {
    createContext: (value) => ({ value }),
    useState(initial) {
      const index = cursor++;
      if (!slots[index])
        slots[index] = {
          value: typeof initial === "function" ? initial() : initial,
        };
      return [
        slots[index].value,
        (update) => {
          const value =
            typeof update === "function" ? update(slots[index].value) : update;
          if (!Object.is(value, slots[index].value)) {
            slots[index].value = value;
            dirty = true;
          }
        },
      ];
    },
    useRef(initial) {
      return react.useState(() => ({ current: initial }))[0];
    },
    useLayoutEffect(effect, deps) {
      react.useEffect(effect, deps);
    },
    useMemo(calculate, deps) {
      const index = cursor++;
      if (!slots[index] || !sameDeps(slots[index].deps, deps))
        slots[index] = { deps, value: calculate() };
      return slots[index].value;
    },
    useCallback(callback, deps) {
      return react.useMemo(() => callback, deps);
    },
    useEffect(effect, deps) {
      const index = cursor++;
      if (!slots[index] || !sameDeps(slots[index].deps, deps)) {
        pendingEffects.push(() => {
          slots[index]?.cleanup?.();
          slots[index] = { deps, effect, cleanup: effect() };
        });
      }
    },
  };
  const makeDeck = () => ({
    get_all_cards_summary: () => [{ due_timestamp_ms: due }],
    get_sentence_list: () => ({ type: "Movie", id: "persisted" }),
    get_total_reviews: () => totalReviews,
    cache_challenge_audio: (banned, token, signal) =>
      prefetches.push({ banned, token, signal }),
    home_screen_view: (inputs) => ({ inputs }),
    review_screen_view: (inputs) => ({
      step: placement
        ? { type: "PlacementTest", view: inputs.placement ?? { progress: 0 } }
        : accomplishment &&
            inputs.dismissed_accomplishment_at_review !== Number(totalReviews)
          ? { type: "Accomplishment" }
          : inputs.current_challenge || selected
            ? {
                type: "Challenge",
                view: { challenge: inputs.current_challenge ?? selected },
              }
            : { type: "Idle", view: { sentence_list: inputs.sentence_list } },
    }),
    translate_sentence_perfect: (tapped, sentence) => eventAvailable ? { sentence, tapped } : undefined,
    translate_sentence_wrong: (sentence, submission) => eventAvailable ? { sentence, submission } : undefined,
    transcribe_sentence: (grade) => eventAvailable ? { grade } : undefined,
    review_card: (indicator, rating) => eventAvailable ? { indicator, rating } : undefined,
  });
  let state = {
    type: "deck",
    deck: makeDeck(),
    targetLanguage: "French",
    historyKnown: true,
  };
  const exports = {};
  class Clock extends Date {
    constructor(...args) {
      super(...(args.length ? args : [now]));
    }
    static now() {
      return now;
    }
  }
  vm.runInNewContext(source, {
    exports,
    console,
    AbortController,
    Date: Clock,
    localStorage,
    window: { scrollTo() {} },
    setTimeout(callback, delay) {
      const id = {};
      timers.set(id, { callback, at: now + delay });
      return id;
    },
    clearTimeout: (id) => timers.delete(id),
    require(path) {
      if (path === "react") return react;
      if (path === "react/jsx-runtime")
        return { jsx: (type, props, key) => ({ type, props, key }) };
      if (path === "react-router-dom")
        return { useOutletContext: () => context };
      if (path === "react-use")
        return {
          useNetworkState: () => ({ online }),
          useInterval: (fn, delay) => {
            assert.equal(delay, 2000);
            poll = fn;
          },
        };
      if (path === "@/hooks/useDeck")
        return {
          useDeckSelection: () => ({
            type: "languageSelected",
            targetLanguage: state.targetLanguage,
            nativeLanguage: "English",
          }),
        };
      if (path === "@/weapon")
        return {
          useWeapon: () => ({
            add_deck_event: (event) => events.push(event),
            add_deck_event_at: (event, at) => events.push({ event, at }),
          }),
        };
      if (path === "@/lib/sound-effects") return { playSoundEffect() {} };
      if (path === "@/lib/challenge-restrictions")
        return {
          readChallengeRestrictions() {
            restrictionReads++;
            const banned = [];
            for (const [kind, requirement] of [
              ["listen", "Listening"],
              ["speak", "Speaking"],
            ]) {
              const key = `yap-cant-${kind}-timestamp`;
              if (storage.has(key) && now - Number(storage.get(key)) < 5000)
                banned.push(requirement);
              else storage.delete(key);
            }
            return { banned };
          },
        };
      assert.ok(path.endsWith("/pkg"), path);
      return {
        get_audio_cache_version: () => audio,
        get_clip_manifest_version: () => clips,
        refresh_clip_manifest: async () => {
          manifestRefreshes++;
        },
      };
    },
  });
  function render() {
    let attempts = 0;
    do {
      assert.ok(++attempts < 20, "render converges before commit");
      dirty = false;
      cursor = 0;
      pendingEffects = [];
      session = exports.useStudyController(state, context, exports.CourseRoutes().props.pendingReviewScope);
    } while (dirty);
    pendingEffects.forEach((run) => run());
    return session;
  }
  return {
    render,
    events,
    prefetches,
    storage,
    get session() {
      return session;
    },
    get restrictionReads() {
      return restrictionReads;
    },
    get manifestRefreshes() {
      return manifestRefreshes;
    },
    get timerTimes() {
      return [...timers.values()].map((timer) => timer.at);
    },
    routeKey: () => exports.CourseRoutes().key,
    setEventAvailable: (value) => { eventAvailable = value; },
    setUser: (id) => {
      context.userInfo = id === undefined ? undefined : { id, displayName: "Learner" };
    },
    setCourse: (course) => {
      state.targetLanguage = course;
    },
    select: (value) => {
      selected = value;
    },
    setDue: (value) => {
      due = value;
    },
    setAccomplishment: (value) => {
      accomplishment = value;
    },
    setPlacement: (value) => {
      placement = value;
    },
    replaceDeck: () => {
      state = { ...state, deck: makeDeck() };
      return render();
    },
    completeReview: () => {
      totalReviews++;
      return render();
    },
    reconnect: () => {
      online = !online;
      return render();
    },
    tick(ms, { audioChanged = false, clipsChanged = false } = {}) {
      now += ms;
      if (audioChanged) audio++;
      if (clipsChanged) clips++;
      poll();
      for (const [id, timer] of timers) {
        if (timer.at <= now) {
          timers.delete(id);
          timer.callback();
        }
      }
      return render();
    },
    replayEffects: () => slots.forEach((slot) => {
      if (slot.effect) { slot.cleanup?.(); slot.cleanup = slot.effect(); }
    }),
    unmount: () => slots.forEach((slot) => slot.cleanup?.()),
  };
}

const challenge = (sentence) => ({
  type: "TranslateComprehensibleSentence",
  target_language: sentence,
});

test("media, minutes, and automatic restriction expiry preserve the held answer and grading target", async () => {
  const h = harness();
  const first = challenge("first sentence");
  h.storage.set("yap-cant-listen-timestamp", "1000000");
  h.select(first);
  h.render();
  h.select(challenge("replacement"));
  const reads = h.restrictionReads;
  h.tick(2000);
  assert.equal(h.restrictionReads, reads + 1);
  h.tick(4000, { audioChanged: true, clipsChanged: true });
  assert.equal(h.session.inputs.banned.length, 0);
  assert.equal(h.session.currentChallenge, first);
  h.tick(60_000);
  assert.equal(h.session.currentChallenge, first);
  await h.session.actions.onTranslationComplete(
    { perfect: null },
    [],
    "typed answer",
    1234,
  );
  assert.equal(h.events[0].event.sentence, "first sentence");
  assert.equal(h.events[0].at, 1234);
});

test("explicit restrictions and undo invalidate immediately; a new snapshot repicks", () => {
  const h = harness();
  h.select(challenge("one"));
  h.render();
  for (const action of ["onCantListen", "onCantSpeak", "undoRestrictions"]) {
    h.select(challenge(action));
    h.session.actions[action]();
    h.render();
    assert.equal(h.session.currentChallenge.target_language, action);
  }
  h.select(challenge("new snapshot"));
  h.replaceDeck();
  assert.equal(h.session.currentChallenge.target_language, "new snapshot");
});

test("idle never sticks, and a distant due timer arms after a minute poll", () => {
  const h = harness();
  h.setDue(1_125_000);
  h.render();
  assert.equal(h.session.currentChallenge, undefined);
  assert.equal(h.timerTimes.length, 0);
  h.tick(60_000);
  assert.equal(h.timerTimes.length, 0);
  h.tick(60_000);
  assert.ok(h.timerTimes.includes(1_125_001));
  h.select(challenge("newly due"));
  h.tick(5001);
  assert.equal(h.session.currentChallenge.target_language, "newly due");
});

test("dismissal and placement survive snapshot replacement; review-count changes reoffer accomplishment", () => {
  const h = harness();
  h.setAccomplishment(true);
  h.render();
  assert.equal(h.session.getReviewView(undefined).step.type, "Accomplishment");
  h.session.actions.dismissAccomplishment();
  h.render();
  h.replaceDeck();
  assert.equal(h.session.getReviewView(undefined).step.type, "Idle");
  h.completeReview();
  assert.equal(h.session.getReviewView(undefined).step.type, "Accomplishment");
  h.setPlacement(true);
  const placement = { progress: 3 };
  h.session.actions.setPlacement(placement);
  h.render();
  h.replaceDeck();
  assert.equal(h.session.getReviewView(undefined).step.view, placement);
});

test("view projection accepts local curriculum selection without retaining the override", () => {
  const h = harness();
  h.render();
  const local = { type: "Movie", id: "screen-local" };
  assert.equal(h.session.getReviewView(local).step.view.sentence_list, local);
  assert.equal(h.session.getHomeView(local).inputs.sentence_list, local);
  assert.equal(h.session.inputs.sentence_list.id, "persisted");
  assert.equal(
    h.session.getReviewView(undefined).step.view.sentence_list,
    undefined,
  );
});

test("course session key ignores deck replacement and resets for user or course changes", () => {
  const h = harness();
  const key = h.routeKey();
  h.replaceDeck();
  assert.equal(h.routeKey(), key);
  h.setUser("two");
  assert.notEqual(h.routeKey(), key);
  const userKey = h.routeKey();
  h.setCourse("Spanish");
  assert.notEqual(h.routeKey(), userKey);
  assert.equal(h.render().actions.pendingReviewScope, "two:Spanish:English");
  h.setUser(undefined);
  assert.equal(h.render().actions.pendingReviewScope, "anon:Spanish:English");
});

for (const [kind, complete] of [
  ["TranslateComprehensibleSentence", (actions) => actions.onTranslationComplete({ perfect: null }, [], "answer", 1234)],
  ["TranslateComprehensibleSentence", (actions) => actions.onTranslationComplete({ literalGrades: [], phrasesRemembered: [], phrasesForgot: [] }, [], "answer", 1234)],
  ["TranscribeComprehensibleSentence", (actions) => actions.onTranscriptionComplete([], 1234)],
  ["FlashCardReview", (actions) => actions.onRating("good")],
  ["PronunciationChallenge", (actions) => actions.onRating("again")],
]) {
  test(`${kind} acknowledges once per deck snapshot and rejects stale callbacks`, () => {
    const h = harness();
    h.select({ type: kind, target_language: "sentence", indicator: "card" });
    const oldActions = h.render().actions;
    h.setEventAvailable(false);
    assert.equal(complete(oldActions), false);
    h.setEventAvailable(true);
    assert.equal(complete(oldActions), true);
    assert.equal(complete(oldActions), false);
    assert.equal(h.events.length, 1);
    h.replayEffects();
    assert.equal(complete(h.session.actions), false, "StrictMode replay must not release the guard");
    h.tick(60_000, { audioChanged: true, clipsChanged: true });
    assert.equal(complete(h.session.actions), false);
    const newActions = h.replaceDeck().actions;
    assert.equal(complete(oldActions), false);
    assert.equal(complete(newActions), true);
    assert.equal(complete(newActions), false);
    assert.equal(h.events.length, 2);
  });
}

test("Home owns prefetch too; readiness/reconnect abort superseded work and unmount cleans up", () => {
  const h = harness();
  h.render();
  assert.equal(h.manifestRefreshes, 1);
  assert.equal(h.prefetches.length, 1);
  h.tick(2000);
  assert.equal(h.prefetches.length, 1);
  h.tick(2000, { clipsChanged: true });
  assert.ok(h.prefetches[0].signal.aborted);
  const previous = h.prefetches.at(-1);
  h.reconnect();
  assert.ok(previous.signal.aborted);
  h.unmount();
  assert.ok(h.prefetches.at(-1).signal.aborted);
});
