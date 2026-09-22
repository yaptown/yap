import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  useSyncExternalStore,
} from "react";
import * as Sentry from "@sentry/react";
import { useAsyncMemo, useWeapon } from "@/weapon";
import type {
  Course,
  Deck,
  Language,
  LanguageDataError,
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

export function useDeck():
  | {
      type: "deck";
      nativeLanguage: Language;
      targetLanguage: Language;
      deck: Deck | null;
      startingFresh: boolean | undefined;
      historyKnown: boolean;
    }
  | { type: "noLanguageSelected" }
  | { type: "error"; message: string; retry: () => void; retryCount: number }
  | { type: "loading"; message: string; progress: number }
  | null {
  const weapon = useWeapon();
  const [retryCount, setRetryCount] = useState(0);
  const [loadingState, setLoadingState] = useState<{
    message: string;
    progress: number;
  } | null>(null);

  useEffect(() => {
    weapon.request_deck_selection();
    weapon.request_reviews();
  }, [weapon]);

  const getSnapshot = useCallback(() => {
    try {
      const num_reviews = weapon.get_stream_num_events("reviews");
      const num_deck_selection = weapon.get_stream_num_events("deck_selection");
      if (num_reviews === undefined || num_deck_selection === undefined) {
        return null;
      }
      return num_reviews + num_deck_selection;
    } catch {
      return null;
    }
  }, [weapon]);

  const subscribe = useCallback(
    (callback: () => void) => {
      const handle_reviews = weapon.subscribe_to_stream("reviews", () => {
        callback();
      });
      const handle_deck_selection = weapon.subscribe_to_stream(
        "deck_selection",
        () => {
          callback();
        },
      );

      return () => {
        weapon.unsubscribe(handle_reviews);
        weapon.unsubscribe(handle_deck_selection);
      };
    },
    [weapon],
  );

  const numEvents = useSyncExternalStore(subscribe, getSnapshot);

  // Whether the reviews stream has been confirmed against the server (or
  // the user is anonymous and local is the whole truth). The placement-test
  // decision reads absence-of-an-event as "never took it", which is only
  // sound once this is true — before then, a fresh device's empty local
  // store would re-offer the test to someone who already took it. Shares
  // the stream subscription: completing a sync marks the stream dirty, so
  // the flip re-renders even when zero events came down.
  const historyKnownSnapshot = useCallback(
    () => weapon.reviews_history_known(),
    [weapon],
  );
  const historyKnown = useSyncExternalStore(subscribe, historyKnownSnapshot);

  const retry = useCallback(() => {
    setRetryCount((count) => count + 1);
  }, []);

  // Determine course: from weapon streams if ready, else from localStorage cache
  const deck_selection = weapon.get_deck_selection_state();
  const courseParts =
    numEvents !== null &&
    deck_selection?.targetLanguage &&
    deck_selection?.nativeLanguage
      ? {
          nativeLanguage: deck_selection.nativeLanguage,
          targetLanguage: deck_selection.targetLanguage,
        }
      : null;
  if (courseParts) {
    localStorage.setItem(LAST_COURSE_KEY, JSON.stringify(courseParts));
  }
  const cachedCourse = useMemo<Course | null>(() => {
    try {
      const cached = localStorage.getItem(LAST_COURSE_KEY);
      if (!cached) return null;
      const parsed = JSON.parse(cached);
      if (parsed.nativeLanguage && parsed.targetLanguage) return parsed;
    } catch {
      /* ignore */
    }
    return null;
  }, []);
  const course = courseParts ?? cachedCourse;
  const courseKey = getCourseKey(course);

  const deckInputsSnapshot = useCallback(
    () => (course ? weapon.deck_inputs_key(course) : null),
    [weapon, course],
  );
  const deckInputsKey = useSyncExternalStore(subscribe, deckInputsSnapshot);
  const streamsReady = numEvents !== null;

  // Fetch language pack — only re-runs when course changes, not when numEvents
  // changes. Two-stage: the core half (dictionary + frequencies) loads first
  // so the placement test can start immediately; when the sentence half lands
  // a fresh result is published and the deck below is rebuilt against it.
  type LanguagePackResult =
    | { courseKey: string; ok: true }
    | { courseKey: string; ok: false; error: unknown };
  const [languagePackResult, setLanguagePackResult] =
    useState<LanguagePackResult | null>(null);
  // Generation counter so a superseded load (course switch, retry) can't
  // clobber the current one's result after the fact — a stale write here
  // would strand the app on the loading screen, since no further updates
  // ever arrive once both loads have finished.
  const packLoadGeneration = useRef(0);
  useAsyncMemo(async () => {
    const generation = ++packLoadGeneration.current;
    const alive = () => packLoadGeneration.current === generation;
    setLanguagePackResult(null);
    if (!course || !courseKey) return null;
    Sentry.addBreadcrumb({
      category: "language-pack",
      message: `Loading language pack: ${course.targetLanguage} → ${course.nativeLanguage}`,
      level: "info",
    });
    const onProgress = (message: string, progress: number) => {
      Sentry.addBreadcrumb({
        category: "language-pack",
        message: `${message} (${Math.round(progress)}%)`,
        level: "info",
      });
      if (alive()) setLoadingState({ message, progress });
    };
    try {
      await weapon.load_language_pack_core(course, onProgress);
    } catch (error) {
      if (!alive()) return null;
      setLoadingState(null);
      setLanguagePackResult({ courseKey, ok: false, error });
      return null;
    }
    if (!alive()) return null;
    setLanguagePackResult({ courseKey, ok: true });
    // The sentence half downloads in the background, possibly while the
    // placement test is already underway. Rust retries transient chunk failures
    // for both halves before surfacing an error to the host.
    try {
      await weapon.load_language_pack(course, onProgress);
    } catch (error) {
      if (!alive()) return null;
      setLoadingState(null);
      setLanguagePackResult({ courseKey, ok: false, error });
      return null;
    }
    if (!alive()) return null;
    setLoadingState(null);
    setLanguagePackResult({ courseKey, ok: true });
    return null;
  }, [weapon, courseKey, retryCount]);

  // Build deck when Rust says its inputs changed (or pack loading reports an error).
  const state = useAsyncMemo(async () => {
    if (!streamsReady) return null;

    if (!deck_selection?.targetLanguage || !deck_selection?.nativeLanguage) {
      return { type: "noLanguageSelected" } as { type: "noLanguageSelected" };
    }

    if (!course || !courseKey || !languagePackResult) return null;
    if (languagePackResult.courseKey !== courseKey) return null;

    if (!languagePackResult.ok) {
      const error = languagePackResult.error;
      console.error("Failed to fetch language pack:", error);
      const errorMessage = getErrorMessage(error);
      const detail = (error as Partial<LanguageDataError> | null)?.detail;
      const isNetworkError =
        detail?.type === "Download" || detail?.type === "Timeout";
      if (!isNetworkError) {
        // Only report non-network errors to Sentry. Network failures are expected
        // on flaky mobile connections and the user already sees a retry UI.
        Sentry.captureException(
          error instanceof Error ? error : new Error(errorMessage),
          {
            tags: {
              "language-pack.target": course.targetLanguage,
              "language-pack.native": course.nativeLanguage,
            },
            contexts: {
              "language-pack": {
                targetLanguage: course.targetLanguage,
                nativeLanguage: course.nativeLanguage,
                rawError: errorMessage,
              },
            },
          },
        );
      }
      return {
        type: "error",
        courseKey,
        message: errorMessage,
        retry,
        retryCount,
      } as {
        type: "error";
        courseKey: string;
        message: string;
        retry: () => void;
        retryCount: number;
      };
    }

    try {
      const deck = await weapon.get_deck_state(
        course,
        new Date().getTimezoneOffset() * -60,
      );

      // null while only the core half of the pack is loaded and this user
      // would not see the placement test; the sentence half publishes a new
      // pack result and this re-runs.
      if (!deck) return null;

      return {
        type: "deck",
        courseKey,
        startingFresh: deck_selection.onboardingSelections?.startingFresh,
        historyKnown,
        nativeLanguage: course.nativeLanguage,
        targetLanguage: course.targetLanguage,
        deck,
      } as {
        type: "deck";
        courseKey: string;
        nativeLanguage: Language;
        targetLanguage: Language;
        deck: Deck | null;
        startingFresh: boolean | undefined;
        historyKnown: boolean;
      };
    } catch (error) {
      const errorMessage = getErrorMessage(error);
      Sentry.captureException(
        error instanceof Error ? error : new Error(errorMessage),
        {
          tags: {
            "language-pack.target": course.targetLanguage,
            "language-pack.native": course.nativeLanguage,
            "language-pack.phase": "deck-state",
          },
          contexts: {
            "language-pack": {
              targetLanguage: course.targetLanguage,
              nativeLanguage: course.nativeLanguage,
              rawError: errorMessage,
            },
          },
        },
      );
      return {
        type: "error",
        courseKey,
        message: errorMessage,
        retry,
        retryCount,
      } as {
        type: "error";
        courseKey: string;
        message: string;
        retry: () => void;
        retryCount: number;
      };
    }
  }, [
    weapon,
    deckInputsKey,
    streamsReady,
    courseKey,
    languagePackResult,
    retryCount,
  ]);

  const currentState =
    state &&
    (state.type === "deck" || state.type === "error") &&
    state.courseKey !== courseKey
      ? null
      : state;

  if (currentState?.type === "error" && currentState.retryCount < retryCount) {
    return null;
  }

  // If we're loading and have progress info, return loading state
  if (loadingState && (currentState === null || currentState === undefined)) {
    return {
      type: "loading",
      message: loadingState.message,
      progress: loadingState.progress,
    };
  }

  return currentState ?? null;
}
