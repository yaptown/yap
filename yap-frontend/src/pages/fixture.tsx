/* eslint-disable no-console -- fixture interactions deliberately do not write review events */
import { useEffect, useState } from "react";
import { useOutletContext, useParams } from "react-router-dom";
import { type AppContextType, useDeck } from "@/App";
import { NoCardsReady } from "@/components/no-cards-ready";
import { AccomplishmentScreen } from "@/components/AccomplishmentScreen";
import { ChallengeView } from "@/components/challenges/ChallengeView";
import type {
  ChallengeFixture,
  Fixture,
  TranscriptionState,
  TranslationState,
} from "../../../yap-frontend-rs/pkg";

// Captures use serde JSON; only reducer inputs need a bridge representation.
function parseFixture(json: string): Fixture {
  const parsed = JSON.parse(json) as Fixture;
  if (parsed.type !== "Challenge") return parsed;
  const fixture = parsed.view as unknown as {
    challenge: ChallengeFixture["challenge"];
    translation?: TranslationState | null;
    transcription:
      | (Omit<TranscriptionState, "inputs"> & {
          inputs: Record<string, string>;
        })
      | null;
  };
  return { type: "Challenge", view: {
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
  } };
}

const log = (...args: unknown[]) => console.log("fixture action", ...args);

export function FixturePage() {
  const { name } = useParams();
  const { accessToken } = useOutletContext<AppContextType>();
  const deckState = useDeck();
  const [loaded, setLoaded] = useState<{
    name: string;
    fixture: Fixture;
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
  const fixture = loaded.fixture;
  return (
    <div data-fixture-rendered={name} className="flex flex-col gap-6">
      {fixture.type === "Idle" ? (
        <NoCardsReady key={name} view={fixture.view} deck={deck} addEvent={log} undoRestrictions={log} setSentenceList={log} showEngagementPrompts={false} />
      ) : fixture.type === "Accomplishment" ? (
        <AccomplishmentScreen key={name} view={fixture.view} addEvent={log} onDismiss={log} />
      ) : <ChallengeView
        key={name}
        challenge={fixture.view.challenge}
        initialState={fixture.view.transcription ?? undefined}
        translationState={fixture.view.translation ?? undefined}
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
      />}
    </div>
  );
}
