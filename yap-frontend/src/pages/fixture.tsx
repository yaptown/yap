/* eslint-disable no-console -- fixture interactions deliberately do not write review events */
import { useEffect, useState } from "react";
import { useOutletContext, useParams } from "react-router-dom";
import { type AppContextType, useDeck } from "@/App";
import { ReviewScreen } from "@/components/ReviewScreen";
import { TopPageLayout } from "@/components/TopPageLayout";
import type {
  ChallengeView,
  ReviewScreenView,
  TranscriptionState,
  TranslationState,
} from "../../../yap-frontend-rs/pkg";

// Captures use serde JSON; only reducer inputs need a bridge representation.
function parseFixture(json: string): ReviewScreenView {
  const parsed = JSON.parse(json) as ReviewScreenView;
  if (parsed.step.type !== "Challenge") return parsed;
  const fixture = parsed.step.view as unknown as {
    challenge: ChallengeView["challenge"];
    translation?: TranslationState | null;
    transcription:
      | (Omit<TranscriptionState, "inputs"> & {
          inputs: Record<string, string>;
        })
      | null;
  };
  return {
    ...parsed,
    step: {
      type: "Challenge",
      view: {
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
      },
    },
  };
}

const log = (...args: unknown[]) => console.log("fixture action", ...args);

export function FixturePage() {
  const { name } = useParams();
  const { accessToken, userInfo } = useOutletContext<AppContextType>();
  const deckState = useDeck();
  const [loaded, setLoaded] = useState<{
    name: string;
    fixture: ReviewScreenView;
  }>();
  const [error, setError] = useState<{
    name: string | undefined;
    message: string;
  }>();
  useEffect(() => {
    const abort = new AbortController();
    void fetch(`/__fixtures/${name}.json`, { signal: abort.signal })
      .then(async (response) => {
        if (!response.ok)
          throw new Error(`ReviewScreenView: ${response.status}`);
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
  const { deck } = deckState;
  const fixture = loaded.fixture;
  return (
    // Mirror ReviewPage's shell so captures show the same header and progress
    // bar the real screen does — iOS renders the whole app, so a bare fixture
    // would make every web/iOS pair differ by the chrome alone. The signup nag
    // stays off: it varies with auth state and would make captures unstable.
    <TopPageLayout
      userInfo={userInfo}
      headerProps={{
        onChangeLanguage: log,
        showSignupNag: false,
        language: fixture.target_language,
        dailyGoalPercent: fixture.progress * 100,
      }}
    >
      {/* Same container Review uses. flex-1 is load-bearing: it stretches this
          to TopPageLayout's 100dvh, which is what gives the challenges'
          `sticky bottom-0` action bars a containing block reaching the viewport
          bottom. Without it they sit directly under the card instead. */}
      <div data-fixture-rendered={name} className="flex flex-col flex-1 gap-2">
        <ReviewScreen
          key={name}
          view={fixture}
          host={{
            deck,
            accessToken,
            autoplayed: true,
            setAutoplayed: log,
          }}
          actions={{
            setPlacement: log,
            onRating: log,
            onTranslationComplete: log,
            onTranscriptionComplete: log,
            onCantListen: log,
            onCantSpeak: log,
            addEvent: log,
            undoRestrictions: log,
            setSentenceList: log,
            dismissAccomplishment: log,
            completePlacementTest: log,
            saveDisplayName: log,
            skipDisplayName: log,
          }}
        />
      </div>
    </TopPageLayout>
  );
}
