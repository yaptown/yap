import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useLayoutEffect,
  useRef,
  useMemo,
  useState,
} from "react";
import { Outlet, useOutletContext } from "react-router-dom";
import { useInterval, useNetworkState } from "react-use";
import type { AppContextType } from "@/app/context";
import { useDeck, useDeckSelection } from "@/core/useDeck";
import { useReportDeckSwitching, useWeapon, useWeaponState } from "@/core/weapon";
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
  type ManualTranslationGrade,
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

// Changing course starts a new session, and so does one signed-in account
// replacing another (its answers must not carry over). Signing in from a
// signed-out session replaces the store, not the mounted screens; visiting the
// language picker also preserves it.
export function CourseRoutes() {
  const context = useOutletContext<AppContextType>();
  const selection = useDeckSelection();
  const user = context.userInfo?.id;
  const [account, setAccount] = useState({ user, epoch: 0 });
  if (account.user !== user) {
    setAccount({ user, epoch: account.user !== undefined && user !== undefined ? account.epoch + 1 : account.epoch });
  }
  const courseKey =
    selection?.type === "languageSelected"
      ? `${selection.targetLanguage}:${selection.nativeLanguage}`
      : "unselected";
  const pendingReviewScope = `${context.userInfo?.id ?? "anon"}:${courseKey}`;
  return (
    <CourseSession
      key={`${account.epoch}:${courseKey}`}
      pendingReviewScope={pendingReviewScope}
      context={context}
    />
  );
}

function CourseSession({ context, pendingReviewScope }: { context: AppContextType; pendingReviewScope: string }) {
  const state = useDeck();
  useReportDeckSwitching(state.switching);
  const study = useStudyController(state, context, pendingReviewScope);
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

// Only mounted Home/Learn screens request and commit a study selection. This
// effect must live in the consumer: navigation need not rerender CourseSession.
// eslint-disable-next-line react-refresh/only-export-components -- The course provider intentionally exposes its consumer hooks together.
export function useStudyReviewView(sentenceList: SentenceListSelection | undefined) {
  const { getReviewView, commitChallenge } = useCourseStudy();
  const view = getReviewView(sentenceList);
  const challenge = view.step.type === "Challenge" ? view.step.view.challenge : undefined;
  useLayoutEffect(() => {
    commitChallenge(challenge);
  }, [commitChallenge, challenge]);
  return view;
}

// Mount only on screens that need study audio; unmounting cancels prefetch.
export function CourseAudioPrefetch() {
  const { deck, accessToken, banned, readiness, online } = useCourseStudy().audioPrefetch;
  useEffect(() => {
    if (!deck) return;
    const controller = new AbortController();
    deck.cache_challenge_audio(banned, accessToken, controller.signal);
    return () => controller.abort();
  }, [deck, accessToken, banned, readiness, online]);
  return null;
}

function useStudyController(
  state: ReturnType<typeof useDeck>,
  { userInfo, accessToken }: AppContextType,
  pendingReviewScope: string,
) {
  const deck = state.view.phase.type === "Ready" ? state.deck : null;
  const submitting = useRef({ deck, inFlight: false });
  // Reset before child resume effects run, and only for a new snapshot (not
  // StrictMode's repeated effect setup). Old snapshot callbacks stay rejected.
  useLayoutEffect(() => {
    if (submitting.current.deck !== deck) submitting.current = { deck, inFlight: false };
  }, [deck]);
  const targetLanguage = deck ? state.course?.targetLanguage : undefined;
  const startingFresh = state.startingFresh;
  const historyKnown = state.historyKnown;
  const weapon = useWeapon();
  const weaponState = useWeaponState();
  const switching = state.switching || (weaponState.type === "ready" && weaponState.switching);
  const activeStore = useRef({ weapon, switching });
  useLayoutEffect(() => {
    activeStore.current = { weapon, switching };
    return () => { activeStore.current.switching = true; };
  }, [weapon, switching]);
  const canWrite = useCallback(
    () => activeStore.current.weapon === weapon && !activeStore.current.switching,
    [weapon],
  );
  const network = useNetworkState();
  const [polledReadiness, setReadiness] = useState(() => ({
    timestamp_ms: Date.now(),
    audio: get_audio_cache_version(),
    clips: get_clip_manifest_version(),
  }));
  // A new immutable deck needs a fresh clock, not a render-phase state update.
  const readiness = useMemo(
    // eslint-disable-next-line react-hooks/purity -- Sample once per immutable snapshot/readiness change, without a second render.
    () => ({ ...polledReadiness, deck, timestamp_ms: Date.now() }),
    [deck, polledReadiness],
  );
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

  const cards = useMemo(() => deck?.get_all_cards_summary(), [deck]);
  const sentenceList = useMemo(() => deck?.get_sentence_list(), [deck]);
  const nextDue = useMemo(
    () =>
      cards?.reduce(
          (next, card) =>
            card.due_timestamp_ms > readiness.timestamp_ms
              ? Math.min(next, card.due_timestamp_ms)
              : next,
          Infinity,
        ) ?? Infinity,
    [cards, readiness.timestamp_ms],
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
    if (!switching && deck && accessToken && userInfo?.id) {
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
  }, [deck, accessToken, userInfo?.id, switching]);

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
  const heldChallenge = useRef<{
    deck: Deck;
    revision: number;
    challenge: Challenge<Gram<string>>;
  }>(undefined);
  const { inputs, getReviewView, getHomeView } = useMemo(() => {
    // Read the last committed selection; only a new deck or an explicit ban
    // releases it. Capturing a new selection happens after commit, without a
    // second render (and therefore without a second Rust screen computation).
    const held = heldChallenge.current;
    const currentHeld = held?.deck === deck && held.revision === restrictionRevision
      ? held.challenge : undefined;
    const reviewInputs = {
      banned,
      sentence_list: sentenceList,
      online: network.online === true,
      is_signed_in: userInfo !== undefined,
      timestamp_ms: readiness.timestamp_ms,
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
    // Creating a controller snapshot never projects a screen. Home and Learn
    // request it on render, sharing one selection and input-keyed caches even
    // when navigation mounts a child without rerendering this controller.
    const reviewViews = new Map<string | undefined, ReturnType<Deck["review_screen_view"]>>();
    const homeViews = new Map<string | undefined, ReturnType<Deck["home_screen_view"]>>();
    // The sentence-list hook intentionally stays screen-local. Rust only uses
    // its selection for curriculum/idle content, not to choose the challenge.
    const getReviewView = (
      sentence_list: SentenceListSelection | undefined,
    ) => {
      if (!deck) throw new Error("Review requires a ready deck");
      // The challenge doesn't depend on the sentence list, so any view already
      // projected decides whether review is idle.
      const reviewView = reviewViews.values().next().value;
      if (reviewView && reviewView.step.type !== "Idle") return reviewView;
      const key = JSON.stringify(sentence_list);
      let view = reviewViews.get(key);
      if (!view) {
        view = deck.review_screen_view({ ...reviewInputs, sentence_list });
        reviewViews.set(key, view);
      }
      return view;
    };
    const getHomeView = (sentence_list: SentenceListSelection | undefined) => {
      if (!deck) throw new Error("Home requires a ready deck");
      const key = JSON.stringify(sentence_list);
      let view = homeViews.get(key);
      if (!view) {
        const review = getReviewView(sentence_list);
        // Home previews precisely the challenge Learn will display, including
        // before the consuming screen's layout effect commits the selection.
        view = deck.home_screen_view({
          ...reviewInputs,
          sentence_list,
          current_challenge: review.step.type === "Challenge"
            ? review.step.view.challenge : reviewInputs.current_challenge,
        });
        homeViews.set(key, view);
      }
      return view;
    };
    return { inputs: reviewInputs, getReviewView, getHomeView };
  }, [
    deck,
    sentenceList,
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
    restrictionRevision,
  ]);
  const commitChallenge = useCallback((challenge: Challenge<Gram<string>> | undefined) => {
    if (deck && challenge) {
      heldChallenge.current = { deck, revision: restrictionRevision, challenge };
    }
  }, [deck, restrictionRevision]);
  // Actions read the committed selection, never force a screen projection.
  // Old snapshot/restriction callbacks cannot submit a replacement challenge.
  const getCurrentChallenge = () => {
    const held = heldChallenge.current;
    return held?.deck === deck && held.revision === restrictionRevision
      ? held.challenge : undefined;
  };
  const totalReviewsCompleted = deck?.get_total_reviews();
  const dismissAccomplishment = useCallback(() => {
    if (totalReviewsCompleted !== undefined)
      setDismissedAccomplishmentAtReview(totalReviewsCompleted);
  }, [totalReviewsCompleted]);
  // Midnight dismisses the previous day's review count even while Home/Learn
  // are unmounted. This needs no screen projection, and prevents a briefly
  // stale accomplishment if navigation precedes the next readiness poll.
  useEffect(() => {
    if (totalReviewsCompleted === undefined || dismissedAccomplishmentAtReview === totalReviewsCompleted) return;
    const now = new Date();
    const midnight = new Date(now);
    midnight.setHours(24, 0, 0, 0);
    const timer = setTimeout(dismissAccomplishment, midnight.getTime() - now.getTime());
    return () => clearTimeout(timer);
  }, [totalReviewsCompleted, dismissedAccomplishmentAtReview, dismissAccomplishment]);

  const addEvent = useCallback(
    (event: DeckEvent) => { if (canWrite()) weapon.add_deck_event(event); },
    [weapon, canWrite],
  );
  const onRating = (rating: Rating): boolean => {
    if (!canWrite() || submitting.current.deck !== deck || submitting.current.inFlight) return false;
    const currentChallenge = getCurrentChallenge();
    if (
      !deck ||
      !currentChallenge ||
      (currentChallenge.type !== "FlashCardReview" &&
        currentChallenge.type !== "PronunciationChallenge")
    ) {
      console.error(
        "handleRating called with no current challenge or incompatible challenge type",
      );
      return false;
    }
    const event = deck.review_card(currentChallenge.indicator, rating);
    if (!event) return false;
    submitting.current.inFlight = true;
    addEvent(event);
    playSoundEffect(rating === "again" ? "fail" : "success");
    window.scrollTo({ top: 0 });
    return true;
  };
  const onTranslationComplete = (
    grade:
      | ManualTranslationGrade
      | { perfect: string | null },
    wordsTapped: number[],
    submission: string,
    completedAtMs: number,
  ): boolean => {
    if (!canWrite() || submitting.current.deck !== deck || submitting.current.inFlight) return false;
    const currentChallenge = getCurrentChallenge();
    if (!deck || currentChallenge?.type !== "TranslateComprehensibleSentence") {
      console.error(
        "handleTranslationComplete called with no current challenge or no TranslateComprehensibleSentence in current challenge",
      );
      return false;
    }
    const event =
      "perfect" in grade
        ? deck.translate_sentence_perfect(
            new Uint32Array(wordsTapped),
            currentChallenge.target_language,
          )
        : deck.translate_sentence_wrong(
            currentChallenge.target_language,
            submission,
            grade.literalGrades,
            new Uint32Array(wordsTapped),
            grade.phrasesRemembered,
            grade.phrasesForgot,
          );
    if (!event) return false;
    submitting.current.inFlight = true;
    weapon.add_deck_event_at(event, completedAtMs);
    playSoundEffect("success");
    window.scrollTo({ top: 0 });
    return true;
  };
  const onTranscriptionComplete = (
    grade: PartGraded[],
    completedAtMs: number,
  ): boolean => {
    if (!canWrite() || submitting.current.deck !== deck || submitting.current.inFlight) return false;
    const currentChallenge = getCurrentChallenge();
    if (
      !deck ||
      currentChallenge?.type !== "TranscribeComprehensibleSentence"
    ) {
      console.error(
        "handleTranscriptionComplete called with no current challenge or no TranscribeComprehensibleSentence in current challenge",
      );
      return false;
    }
    const event = deck.transcribe_sentence(grade);
    if (!event) return false;
    submitting.current.inFlight = true;
    weapon.add_deck_event_at(event, completedAtMs);
    playSoundEffect("success");
    window.scrollTo({ top: 0 });
    return true;
  };
  const restrict = (kind: "listen" | "speak") => {
    localStorage.setItem(`yap-cant-${kind}-timestamp`, Date.now().toString());
    refreshRestrictions();
    heldChallenge.current = undefined;
    setRestrictionRevision((revision) => revision + 1);
    refresh();
  };
  const undoRestrictions = () => {
    localStorage.removeItem("yap-cant-listen-timestamp");
    localStorage.removeItem("yap-cant-speak-timestamp");
    refreshRestrictions();
    heldChallenge.current = undefined;
    setRestrictionRevision((revision) => revision + 1);
    refresh();
  };

  return {
    audioPrefetch: { deck, accessToken, banned, readiness, online: network.online },
    inputs,
    getReviewView,
    getHomeView,
    commitChallenge,
    actions: {
      pendingReviewScope,
      setPlacement,
      addEvent,
      onRating,
      onTranslationComplete,
      onTranscriptionComplete,
      onCantListen: () => restrict("listen"),
      onCantSpeak: () => restrict("speak"),
      undoRestrictions,
      dismissAccomplishment,
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
