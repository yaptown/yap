/* eslint-disable no-console -- fixture interactions deliberately do not write review events */
import { useEffect, useState } from "react";
import { useOutletContext, useParams } from "react-router-dom";
import { type AppContextType, useDeck } from "@/App";
import { ChallengeView } from "@/components/challenges/ChallengeView";
import type {
  ChallengeFixture,
  TranscriptionState,
  TranslationState,
} from "../../../yap-frontend-rs/pkg";

// Captures use serde JSON; only reducer inputs need a bridge representation.
function parseFixture(json: string): ChallengeFixture {
  const fixture = JSON.parse(json) as {
    challenge: ChallengeFixture["challenge"];
    translation?: TranslationState | null;
    transcription:
      | (Omit<TranscriptionState, "inputs"> & {
          inputs: Record<string, string>;
        })
      | null;
  };
  return {
    challenge: fixture.challenge,
    translation: fixture.translation ?? undefined,
    transcription: fixture.transcription
      ? {
          ...fixture.transcription,
          inputs: new Map(
            Object.entries(fixture.transcription.inputs).map(
              ([index, text]) => [Number(index), text],
            ),
          ),
        }
      : undefined,
  };
}

const log = (...args: unknown[]) => console.log("fixture action", ...args);

export function FixturePage() {
  const { name } = useParams();
  const { accessToken } = useOutletContext<AppContextType>();
  const deckState = useDeck();
  const [loaded, setLoaded] = useState<{
    name: string;
    fixture: ChallengeFixture;
  }>();
  const [error, setError] = useState<{
    name: string | undefined;
    message: string;
  }>();
  useEffect(() => {
    const abort = new AbortController();
    void fetch(`/__fixtures/${name}.json`, { signal: abort.signal })
      .then(async (response) => {
        if (!response.ok) throw new Error(`Fixture: ${response.status}`);
        const fixture = parseFixture(await response.text());
        if (!abort.signal.aborted) setLoaded({ name: name!, fixture });
      })
      .catch((error) => {
        if (!abort.signal.aborted) setError({ name, message: String(error) });
      });
    return () => abort.abort();
  }, [name]);
  if (error?.name === name) return <p>{error?.message}</p>;
  if (
    loaded?.name !== name ||
    !loaded ||
    deckState?.type !== "deck" ||
    !deckState.deck
  )
    return <p>Loading fixture…</p>;
  const { deck, targetLanguage, nativeLanguage } = deckState;
  return (
    <div data-fixture-rendered={name} className="flex flex-col gap-6">
      <ChallengeView
        key={name}
        challenge={loaded.fixture.challenge}
        initialState={loaded.fixture.transcription ?? undefined}
        translationState={loaded.fixture.translation ?? undefined}
        deck={deck}
        targetLanguage={targetLanguage}
        nativeLanguage={nativeLanguage}
        accessToken={accessToken}
        totalCount={deck.get_all_cards_summary().length}
        totalReviewsCompleted={deck.get_total_reviews()}
        autoplayed={true}
        setAutoplayed={log}
        onRating={log}
        onTranslationComplete={log}
        onTranscriptionComplete={log}
        onCantListen={log}
        onCantSpeak={log}
      />
    </div>
  );
}
