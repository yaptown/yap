import { useCallback, useEffect, useMemo, useState } from "react";
import { useInterval, useNetworkState } from "react-use";
import {
  get_audio_cache_version,
  get_clip_manifest_version,
  type Deck,
  type HomeScreenInputs,
} from "../../../yap-frontend-rs/pkg";
import { readChallengeRestrictions } from "@/lib/challenge-restrictions";
import { sentenceListToSelection, useSentenceList } from "./useSentenceList";

// Like Review, refresh when the deck changes, audio arrives, or cards become due.
export function useStudyScreenInputs(
  deck: Deck,
  isSignedIn: boolean,
  enabled = true,
): { inputs: HomeScreenInputs; refresh: () => void } {
  const network = useNetworkState();
  const deckSentenceList = useMemo(() => deck.get_sentence_list(), [deck]);
  const { sentenceList } = useSentenceList(deckSentenceList);
  const [readiness, setReadiness] = useState(() => ({
    deck,
    audio: get_audio_cache_version(),
    clips: get_clip_manifest_version(),
    timestamp_ms: Date.now(),
  }));
  if (readiness.deck !== deck) {
    setReadiness((previous) => ({
      ...previous,
      deck,
      timestamp_ms: Date.now(),
    }));
  }
  useInterval(() => {
    const audio = get_audio_cache_version();
    const clips = get_clip_manifest_version();
    const timestamp_ms = Date.now();
    setReadiness((previous) =>
      previous.audio !== audio ||
      previous.clips !== clips ||
      timestamp_ms - previous.timestamp_ms >= 60_000
        ? { ...previous, audio, clips, timestamp_ms }
        : previous,
    );
  }, enabled ? 2000 : null);

  // Fixture views must not expire persisted restrictions or poll a live deck.
  const restrictions = useMemo(
    () =>
      enabled
        ? readChallengeRestrictions(readiness.timestamp_ms)
        : { banned: [], next_expiry_ms: undefined },
    [enabled, readiness.timestamp_ms],
  );
  const nextDue = useMemo(
    () =>
      enabled
        ? deck.get_all_cards_summary().reduce(
            (next, card) =>
              card.due_timestamp_ms > readiness.timestamp_ms
                ? Math.min(next, card.due_timestamp_ms)
                : next,
            Infinity,
          )
        : Infinity,
    [deck, enabled, readiness.timestamp_ms],
  );
  const nextRefresh = Math.min(
    nextDue,
    restrictions.next_expiry_ms ?? Infinity,
  );
  useEffect(() => {
    if (!Number.isFinite(nextRefresh)) return;
    const delay = nextRefresh - Date.now();
    // The minute tick handles distant cards without overflowing setTimeout.
    if (delay > 60_000) return;
    const timer = setTimeout(
      () =>
        setReadiness((previous) => ({ ...previous, timestamp_ms: Date.now() })),
      Math.max(0, delay) + 1,
    );
    return () => clearTimeout(timer);
  }, [nextRefresh, readiness.timestamp_ms]);

  const inputs = useMemo(
    () => ({
      banned: restrictions.banned,
      sentence_list: sentenceListToSelection(sentenceList),
      online: network.online === true,
      is_signed_in: isSignedIn,
      timestamp_ms: readiness.timestamp_ms,
    }),
    [
      restrictions.banned,
      sentenceList,
      network.online,
      isSignedIn,
      readiness.timestamp_ms,
    ],
  );

  // Recompute now instead of waiting for the poll — e.g. after clearing a
  // challenge restriction, so its notice disappears immediately.
  const refresh = useCallback(
    () => setReadiness((previous) => ({ ...previous, timestamp_ms: Date.now() })),
    [],
  );

  return { inputs, refresh };
}
