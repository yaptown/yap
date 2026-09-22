import {
  useCallback,
  useEffect,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
  useSyncExternalStore,
} from "react";
import * as Sentry from "@sentry/react";
import { useWeapon } from "@/core/weapon";
import type {
  Course,
  Deck,
  Language,
  LanguageDataError,
  DeckLoadState,
  DeckLoadEvent,
  DeckLoadView,
} from "../../../yap-frontend-rs/pkg";

import {
  deck_load_start,
  deck_load_transition,
  deck_load_view,
} from "../../../yap-frontend-rs/pkg";

export function useDeckSelection():
  | {
      type: "languageSelected";
      nativeLanguage: Language;
      targetLanguage: Language;
      startingFresh: boolean | undefined;
      hasHeardAbout: boolean;
      onboardedLanguages: Language[];
    }
  | {
      type: "noLanguageSelected";
      hasHeardAbout: boolean;
      onboardedLanguages: Language[];
    }
  | null {
  const weapon = useWeapon();

  useEffect(() => {
    weapon.request_deck_selection();
  }, [weapon]);

  const getSnapshot = useCallback(() => {
    try {
      return weapon.get_stream_num_events("deck_selection") ?? null;
    } catch {
      return null;
    }
  }, [weapon]);

  const subscribe = useCallback(
    (callback: () => void) => {
      const handle = weapon.subscribe_to_stream("deck_selection", () => {
        callback();
      });
      return () => {
        weapon.unsubscribe(handle);
      };
    },
    [weapon],
  );

  const numEvents = useSyncExternalStore(subscribe, getSnapshot);

  if (numEvents === null) return null;

  const deckSelection = weapon.get_deck_selection_state();
  const hasHeardAbout = deckSelection?.heardAbout != null;
  const onboardedLanguages = deckSelection?.onboardedLanguages ?? [];

  if (!deckSelection?.targetLanguage || !deckSelection?.nativeLanguage) {
    return { type: "noLanguageSelected", hasHeardAbout, onboardedLanguages };
  }

  return {
    type: "languageSelected",
    nativeLanguage: deckSelection.nativeLanguage,
    targetLanguage: deckSelection.targetLanguage,
    startingFresh: deckSelection.onboardingSelections?.startingFresh,
    hasHeardAbout,
    onboardedLanguages,
  };
}

const LAST_COURSE_KEY = "yap-last-course";

function getCourseKey(
  course: Pick<Course, "nativeLanguage" | "targetLanguage"> | null | undefined,
): string | null {
  if (!course) return null;
  return `${course.targetLanguage}:${course.nativeLanguage}`;
}

function getErrorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export function useDeck(): {
  view: DeckLoadView;
  deck: Deck | null;
  course: Course | null;
  startingFresh: boolean | undefined;
  historyKnown: boolean;
  retry: () => void;
} {
  const weapon = useWeapon();
  const [snapshot, setSnapshot] = useState<{
    load: DeckLoadState;
    deck: Deck | null;
  }>(() => ({ load: deck_load_start(), deck: null }));
  const dispatchRef = useRef<(event: DeckLoadEvent) => void>(() => {});
  const selectedCourse = useRef<Course | null>(null);

  useEffect(() => {
    weapon.request_deck_selection();
    weapon.request_reviews();
  }, [weapon]);

  const getSnapshot = useCallback(() => {
    try {
      const reviews = weapon.get_stream_num_events("reviews");
      const selection = weapon.get_stream_num_events("deck_selection");
      return reviews === undefined || selection === undefined
        ? null
        : reviews + selection;
    } catch {
      return null;
    }
  }, [weapon]);
  const subscribe = useCallback(
    (callback: () => void) => {
      const reviews = weapon.subscribe_to_stream("reviews", callback);
      const selection = weapon.subscribe_to_stream("deck_selection", callback);
      return () => {
        weapon.unsubscribe(reviews);
        weapon.unsubscribe(selection);
      };
    },
    [weapon],
  );
  const numEvents = useSyncExternalStore(subscribe, getSnapshot);
  const historyKnownSnapshot = useCallback(
    () => weapon.reviews_history_known(),
    [weapon],
  );
  const historyKnown = useSyncExternalStore(subscribe, historyKnownSnapshot);
  const selection = weapon.get_deck_selection_state();
  const nativeLanguage = selection?.nativeLanguage;
  const targetLanguage = selection?.targetLanguage;
  const cachedCourse = useMemo<Course | null>(() => {
    try {
      const parsed = JSON.parse(
        localStorage.getItem(LAST_COURSE_KEY) ?? "null",
      );
      return parsed?.nativeLanguage && parsed?.targetLanguage ? parsed : null;
    } catch {
      return null;
    }
  }, []);
  // Once the streams are ready, an empty selection supersedes the cached course.
  const course = useMemo(
    () =>
      numEvents === null
        ? cachedCourse
        : nativeLanguage && targetLanguage
          ? { nativeLanguage, targetLanguage }
          : null,
    [numEvents, nativeLanguage, targetLanguage, cachedCourse],
  );
  const courseKey = getCourseKey(course);
  const streamsReady = numEvents !== null;
  const languageSelected = !!(nativeLanguage && targetLanguage);
  const deckInputsSnapshot = useCallback(
    () => (course ? weapon.deck_inputs_key(course) : null),
    [weapon, course],
  );
  const deckInputsKey = useSyncExternalStore(subscribe, deckInputsSnapshot);

  // One native execution lifetime. StrictMode cleanup cancels it, and replay
  // starts a fresh reducer so same-course dedup cannot strand a cancelled load.
  useLayoutEffect(() => {
    let active = true;
    let load = deck_load_start();
    let deck: Deck | null = null;
    let packGeneration = 0;
    let buildGeneration = 0;
    let pendingInputs: string | null = null;
    const dispatch = (event: DeckLoadEvent) => {
      if (!active) return;
      if (
        event.type === "CourseChanged" &&
        (event.key ?? null) !== (load.course_key ?? null)
      ) {
        ++packGeneration;
      }
      // Also invalidate B when inputs revert to already-built A: Rust correctly
      // emits no BuildDeck for A, but B must not be allowed to replace it later.
      if (event.type === "InputsChanged" && event.key !== pendingInputs) {
        ++buildGeneration;
        pendingInputs = null;
      }
      const step = deck_load_transition(load, event);
      load = step.state;
      for (const effect of step.effects) {
        switch (effect.type) {
          case "ClearDeck":
            ++buildGeneration;
            pendingInputs = null;
            deck = null;
            break;
          case "LoadPack": {
            const selected = selectedCourse.current;
            if (!selected || getCourseKey(selected) !== effect.course_key)
              break;
            const generation = ++packGeneration;
            const alive = () => active && generation === packGeneration;
            Sentry.addBreadcrumb({
              category: "language-pack",
              message:
                "Loading language pack: " +
                selected.targetLanguage +
                " → " +
                selected.nativeLanguage,
              level: "info",
            });
            const onProgress = (message: string, percent: number) => {
              if (!alive()) return;
              Sentry.addBreadcrumb({
                category: "language-pack",
                message: message + " (" + Math.round(percent) + "%)",
                level: "info",
              });
              dispatch({ type: "PackProgress", message, percent });
            };
            void (async () => {
              try {
                if (effect.from.type === "None") {
                  await weapon.load_language_pack_core(selected, onProgress);
                  if (!alive()) return;
                  dispatch({ type: "CoreLoaded" });
                }
                await weapon.load_language_pack(selected, onProgress);
                if (alive()) dispatch({ type: "FullLoaded" });
              } catch (error) {
                if (!alive()) return;
                const detail = (error as Partial<LanguageDataError> | null)
                  ?.detail;
                dispatch({
                  type: "PackFailed",
                  message: getErrorMessage(error),
                  network:
                    detail?.type === "Download" || detail?.type === "Timeout",
                });
              }
            })();
            break;
          }
          case "RefreshInputs": {
            const selected = selectedCourse.current;
            if (selected)
              dispatch({
                type: "InputsChanged",
                key: weapon.deck_inputs_key(selected),
              });
            break;
          }
          case "BuildDeck": {
            const selected = selectedCourse.current;
            if (!selected || pendingInputs === effect.inputs) break;
            const inputs = effect.inputs;
            pendingInputs = inputs;
            const generation = ++buildGeneration;
            const alive = () =>
              active &&
              generation === buildGeneration &&
              weapon.deck_inputs_key(selected) === inputs;
            void (async () => {
              try {
                const next = await weapon.get_deck_state(
                  selected,
                  new Date().getTimezoneOffset() * -60,
                );
                if (!alive()) return;
                pendingInputs = null;
                deck = next ?? null;
                dispatch({ type: "DeckBuilt", present: deck !== null, inputs });
              } catch (error) {
                if (!alive()) return;
                pendingInputs = null;
                dispatch({
                  type: "DeckBuildFailed",
                  message: getErrorMessage(error),
                });
              }
            })();
            break;
          }
          case "ReportError": {
            if (effect.phase === "pack" && effect.network) break;
            const selected = selectedCourse.current;
            Sentry.captureException(new Error(effect.message), {
              tags: {
                "language-pack.target": selected?.targetLanguage,
                "language-pack.native": selected?.nativeLanguage,
                "language-pack.phase": effect.phase,
              },
              contexts: {
                "language-pack": {
                  targetLanguage: selected?.targetLanguage,
                  nativeLanguage: selected?.nativeLanguage,
                  rawError: effect.message,
                },
              },
            });
            break;
          }
        }
      }
      setSnapshot({ load, deck });
    };
    dispatchRef.current = dispatch;
    setSnapshot({ load, deck });
    return () => {
      active = false;
      ++packGeneration;
      ++buildGeneration;
      dispatchRef.current = () => {};
    };
  }, [weapon]);

  useLayoutEffect(() => {
    selectedCourse.current = course;
    if (streamsReady && course)
      localStorage.setItem(LAST_COURSE_KEY, JSON.stringify(course));
    const dispatch = dispatchRef.current;
    dispatch({ type: "CourseChanged", key: courseKey ?? undefined });
    if (streamsReady)
      dispatch({ type: "StreamsReady", language_selected: languageSelected });
    // Core may already be cached by the time the event streams become ready.
    if (deckInputsKey !== null)
      dispatch({ type: "InputsChanged", key: deckInputsKey });
  }, [
    weapon,
    course,
    courseKey,
    streamsReady,
    languageSelected,
    deckInputsKey,
  ]);

  const retry = useCallback(() => dispatchRef.current({ type: "Retry" }), []);
  const current =
    (snapshot.load.course_key ?? null) === courseKey
      ? snapshot
      : { load: deck_load_start(), deck: null };
  return {
    view: deck_load_view(current.load),
    deck: current.deck,
    course,
    startingFresh: selection?.onboardingSelections?.startingFresh,
    historyKnown,
    retry,
  };
}
