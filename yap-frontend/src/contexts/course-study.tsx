import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
} from "react";
import { Outlet, useOutletContext } from "react-router-dom";
import { useInterval, useNetworkState } from "react-use";
import type { AppContextType } from "@/App";
import { useDeck, useDeckSelection } from "@/hooks/useDeck";
import { useWeapon } from "@/weapon";
import { readChallengeRestrictions } from "@/lib/challenge-restrictions";
import { playSoundEffect } from "@/lib/sound-effects";
import {
  get_audio_cache_version,
  get_clip_manifest_version,
  refresh_clip_manifest,
  update_profile,
  type Challenge,
  type ChallengeRequirements,
  type Deck,
  type DeckEvent,
  type Gram,
  type Heteronym,
  type LiteralGrades,
  type PartGraded,
  type PlacementSession,
  type Rating,
  type SentenceListSelection,
} from "../../../yap-frontend-rs/pkg";

const DeckContext = createContext<ReturnType<typeof useDeck> | undefined>(
  undefined,
);
const StudyContext = createContext<ReturnType<
  typeof useStudyController
> | null>(null);

// The key is the session identity, never the Deck snapshot or the current route.
// In particular, visiting the language picker and resuming this course preserves it.
export function CourseRoutes() {
  const context = useOutletContext<AppContextType>();
  const selection = useDeckSelection();
  const courseKey =
    selection?.type === "languageSelected"
      ? `${selection.targetLanguage}:${selection.nativeLanguage}`
      : "unselected";
  return (
    <CourseSession
      key={`${context.userInfo?.id ?? "anonymous"}:${courseKey}`}
      context={context}
    />
  );
}

function CourseSession({ context }: { context: AppContextType }) {
  const state = useDeck();
  const study = useStudyController(state, context);
  return (
    <DeckContext.Provider value={state}>
      <StudyContext.Provider value={study}>
        <Outlet context={context} />
      </StudyContext.Provider>
    </DeckContext.Provider>
  );
}

export function useCourseDeck() {
  const state = useContext(DeckContext);
  if (state === undefined)
    throw new Error("Live deck screens require CourseRoutes");
  return state;
}

export function useOptionalCourseStudy() {
  return useContext(StudyContext);
}

export function useCourseStudy() {
  const study = useOptionalCourseStudy();
  if (!study) throw new Error("Live study screens require CourseRoutes");
  return study;
}

function useStudyController(
  state: ReturnType<typeof useDeck>,
  { userInfo, accessToken }: AppContextType,
) {
  const deck = state?.type === "deck" ? state.deck : null;
  const targetLanguage =
    state?.type === "deck" ? state.targetLanguage : undefined;
  const startingFresh =
    state?.type === "deck" ? state.startingFresh : undefined;
  const historyKnown = state?.type === "deck" && state.historyKnown;
  const weapon = useWeapon();
  const network = useNetworkState();
  const [readiness, setReadiness] = useState(() => ({
    deck,
    timestamp_ms: Date.now(),
    audio: get_audio_cache_version(),
    clips: get_clip_manifest_version(),
  }));
  if (readiness.deck !== deck) {
    setReadiness((previous) => ({
      ...previous,
      deck,
      timestamp_ms: Date.now(),
    }));
  }
  const [banned, setBanned] = useState<ChallengeRequirements[]>(
    () => readChallengeRestrictions().banned,
  );
  const refreshRestrictions = useCallback(() => {
    const next = readChallengeRestrictions().banned;
    setBanned((previous) =>
      previous.length === next.length &&
      previous.every((value, i) => value === next[i])
        ? previous
        : next,
    );
  }, []);
  const refresh = useCallback(() => {
    setReadiness((previous) => ({ ...previous, timestamp_ms: Date.now() }));
  }, []);

  // Read restrictions on EVERY poll, even when media versions and the minute
  // haven't changed. Expiry updates scheduling, but never releases a held answer.
  useInterval(() => {
    refreshRestrictions();
    const timestamp_ms = Date.now();
    const audio = get_audio_cache_version();
    const clips = get_clip_manifest_version();
    setReadiness((previous) =>
      previous.audio !== audio ||
      previous.clips !== clips ||
      timestamp_ms - previous.timestamp_ms >= 60_000
        ? { ...previous, timestamp_ms, audio, clips }
        : previous,
    );
  }, 2000);

  const nextDue = useMemo(
    () =>
      deck
        ?.get_all_cards_summary()
        .reduce(
          (next, card) =>
            card.due_timestamp_ms > readiness.timestamp_ms
              ? Math.min(next, card.due_timestamp_ms)
              : next,
          Infinity,
        ) ?? Infinity,
    [deck, readiness.timestamp_ms],
  );
  useEffect(() => {
    if (!Number.isFinite(nextDue)) return;
    const delay = nextDue - Date.now();
    if (delay > 60_000) return;
    const timer = setTimeout(refresh, Math.max(0, delay) + 1);
    return () => clearTimeout(timer);
  }, [nextDue, refresh, readiness.timestamp_ms]);

  useEffect(() => {
    if (!targetLanguage) return;
    refresh_clip_manifest(targetLanguage, accessToken).catch((error) => {
      console.warn("Failed to refresh clip manifest:", error);
    });
  }, [targetLanguage, accessToken]);

  useEffect(() => {
    if (!deck) return;
    const controller = new AbortController();
    deck.cache_challenge_audio(banned, accessToken, controller.signal);
    return () => controller.abort();
  }, [deck, accessToken, banned, readiness, network.online]);

  useEffect(() => {
    if (deck && accessToken && userInfo?.id) {
      deck
        .submit_push_notifications(accessToken, userInfo.id)
        .catch((error) =>
          console.error("Failed to update notification schedule:", error),
        );
      deck
        .submit_language_stats(accessToken)
        .catch((error) =>
          console.error("Failed to update language stats:", error),
        );
    }
  }, [deck, accessToken, userInfo?.id]);

  const [dismissedSetDisplayName, setDismissedSetDisplayName] = useState(
    () => localStorage.getItem("yap-skipped-set-display-name") === "true",
  );
  const [dismissedAccomplishmentAtReview, setDismissedAccomplishmentAtReview] =
    useState<bigint | null>(null);
  const [placement, setPlacement] = useState<PlacementSession>();

  // A challenge, once on screen, stays until the deck itself changes.
  // reviewInfo recomputes underneath for many reasons (a clip manifest
  // arriving, cards becoming due, prefetched audio landing) and can pick a
  // different sentence for the same card — swapping it mid-answer would
  // strand the user's typed input (and their grade) against the wrong
  // sentence. The deck object is rebuilt exactly when events land
  // (completing a review, accepting a lockup offer, a remote sync), which
  // are the moments a re-pick is legitimate; the user changing challenge
  // restrictions mid-challenge (the can't-listen/can't-speak buttons) must
  // also swap immediately. A held "no challenge" never sticks, so newly due
  // cards still surface from idle.
  const [restrictionRevision, setRestrictionRevision] = useState(0);
  const [heldChallenge, setHeldChallenge] = useState<{
    deck: Deck;
    revision: number;
    challenge: Challenge<Gram<string>>;
  }>();
  const currentHeld =
    heldChallenge?.deck === deck &&
    heldChallenge.revision === restrictionRevision
      ? heldChallenge.challenge
      : undefined;
  const { inputs, reviewView, getReviewView, getHomeView } = useMemo(() => {
    const inputs = {
      banned,
      sentence_list: deck?.get_sentence_list(),
      online: network.online === true,
      is_signed_in: userInfo !== undefined,
      timestamp_ms: readiness.timestamp_ms,
    };
    const reviewInputs = {
      ...inputs,
      needs_display_name: userInfo?.displayName === null,
      display_name_dismissed: dismissedSetDisplayName,
      has_access_token: accessToken !== undefined,
      starting_fresh: startingFresh,
      history_known: historyKnown,
      dismissed_accomplishment_at_review:
        dismissedAccomplishmentAtReview === null
          ? undefined
          : Number(dismissedAccomplishmentAtReview),
      placement,
      current_challenge: currentHeld,
    };
    const reviewView = deck?.review_screen_view(reviewInputs);
    // The sentence-list hook intentionally stays screen-local. Rust only uses
    // its selection for curriculum/idle content, not to choose the challenge.
    const getReviewView = (
      sentence_list: SentenceListSelection | undefined,
    ) => {
      if (!deck || !reviewView) throw new Error("Review requires a ready deck");
      return reviewView.step.type === "Idle"
        ? deck.review_screen_view({ ...reviewInputs, sentence_list })
        : reviewView;
    };
    const getHomeView = (sentence_list: SentenceListSelection | undefined) => {
      if (!deck) throw new Error("Home requires a ready deck");
      return deck.home_screen_view({ ...inputs, sentence_list });
    };
    return { inputs, reviewView, getReviewView, getHomeView };
  }, [
    deck,
    banned,
    network.online,
    userInfo,
    readiness,
    dismissedSetDisplayName,
    accessToken,
    startingFresh,
    historyKnown,
    dismissedAccomplishmentAtReview,
    placement,
    currentHeld,
  ]);
  const currentChallenge =
    reviewView?.step.type === "Challenge"
      ? reviewView.step.view.challenge
      : undefined;
  // Adjust before commit, not in an effect; undefined is never a held selection.
  if (deck && currentChallenge && !currentHeld) {
    setHeldChallenge({
      deck,
      revision: restrictionRevision,
      challenge: currentChallenge,
    });
  }

  const totalReviewsCompleted = deck?.get_total_reviews();
  useEffect(() => {
    if (
      reviewView?.step.type !== "Accomplishment" ||
      totalReviewsCompleted === undefined ||
      dismissedAccomplishmentAtReview === totalReviewsCompleted
    )
      return;
    const now = new Date();
    const midnight = new Date(now);
    midnight.setHours(24, 0, 0, 0);
    const timer = setTimeout(
      () => setDismissedAccomplishmentAtReview(totalReviewsCompleted),
      midnight.getTime() - now.getTime(),
    );
    return () => clearTimeout(timer);
  }, [
    reviewView?.step.type,
    dismissedAccomplishmentAtReview,
    totalReviewsCompleted,
  ]);

  const addEvent = useCallback(
    (event: DeckEvent) => weapon.add_deck_event(event),
    [weapon],
  );
  const onRating = async (rating: Rating) => {
    if (
      !deck ||
      !currentChallenge ||
      (currentChallenge.type !== "FlashCardReview" &&
        currentChallenge.type !== "PronunciationChallenge")
    ) {
      console.error(
        "handleRating called with no current challenge or incompatible challenge type",
      );
      return;
    }
    playSoundEffect(rating === "again" ? "fail" : "success");
    window.scrollTo({ top: 0 });
    const event = deck.review_card(currentChallenge.indicator, rating);
    if (event) addEvent(event);
  };
  const onTranslationComplete = async (
    grade:
      | {
          literalGrades: LiteralGrades;
          phrasesRemembered: Gram<string>[];
          phrasesForgot: Gram<string>[];
        }
      | { perfect: string | null },
    wordsTapped: Heteronym<string>[],
    submission: string,
    completedAtMs: number,
  ) => {
    if (!deck || currentChallenge?.type !== "TranslateComprehensibleSentence") {
      console.error(
        "handleTranslationComplete called with no current challenge or no TranslateComprehensibleSentence in current challenge",
      );
      return;
    }
    playSoundEffect("success");
    window.scrollTo({ top: 0 });
    const event =
      "perfect" in grade
        ? deck.translate_sentence_perfect(
            wordsTapped,
            currentChallenge.target_language,
          )
        : deck.translate_sentence_wrong(
            currentChallenge.target_language,
            submission,
            grade.literalGrades,
            wordsTapped,
            grade.phrasesRemembered,
            grade.phrasesForgot,
          );
    if (event) weapon.add_deck_event_at(event, completedAtMs);
  };
  const onTranscriptionComplete = (
    grade: PartGraded[],
    completedAtMs: number,
  ) => {
    if (
      !deck ||
      currentChallenge?.type !== "TranscribeComprehensibleSentence"
    ) {
      console.error(
        "handleTranscriptionComplete called with no current challenge or no TranscribeComprehensibleSentence in current challenge",
      );
      return;
    }
    playSoundEffect("success");
    window.scrollTo({ top: 0 });
    const event = deck.transcribe_sentence(grade);
    if (event) weapon.add_deck_event_at(event, completedAtMs);
  };
  const restrict = (kind: "listen" | "speak") => {
    localStorage.setItem(`yap-cant-${kind}-timestamp`, Date.now().toString());
    refreshRestrictions();
    setRestrictionRevision((revision) => revision + 1);
    refresh();
  };
  const undoRestrictions = () => {
    localStorage.removeItem("yap-cant-listen-timestamp");
    localStorage.removeItem("yap-cant-speak-timestamp");
    refreshRestrictions();
    setRestrictionRevision((revision) => revision + 1);
    refresh();
  };

  return {
    inputs,
    getReviewView,
    getHomeView,
    currentChallenge,
    actions: {
      setPlacement,
      addEvent,
      onRating,
      onTranslationComplete,
      onTranscriptionComplete,
      onCantListen: () => restrict("listen"),
      onCantSpeak: () => restrict("speak"),
      undoRestrictions,
      dismissAccomplishment: () => {
        if (totalReviewsCompleted !== undefined)
          setDismissedAccomplishmentAtReview(totalReviewsCompleted);
      },
      completePlacementTest: ({
        known_words,
        unknown_words,
      }: PlacementSession) => {
        if (deck)
          addEvent(deck.complete_placement_test(known_words, unknown_words));
      },
      saveDisplayName: async (name: string) => {
        await update_profile(name, null, accessToken!);
        setDismissedSetDisplayName(true);
      },
      skipDisplayName: () => {
        localStorage.setItem("yap-skipped-set-display-name", "true");
        setDismissedSetDisplayName(true);
      },
    },
  };
}
