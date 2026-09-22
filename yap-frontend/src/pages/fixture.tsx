/* eslint-disable no-console -- fixture interactions deliberately do not write review events */
import { useEffect, useState } from "react";
import { useOutletContext, useParams } from "react-router-dom";
import { DeckLoadStatus } from "@/app/DeckPage";
import type { AppContextType } from "@/app/context";
import { useDeck } from "@/core/useDeck";
import { ReviewScreen } from "@/review/ReviewScreen";
import { TopPageLayout } from "@/components/TopPageLayout";
import { HomeScreen } from "../browse/HomeScreen";
import { StatsScreen } from "../browse/StatsScreen";
import { GoalsScreen } from "../browse/GoalsScreen";
import { DueWordsScreen } from "../browse/DueWordsScreen";
import type {
  ChallengeView,
  Fixture,
  TranscriptionState,
  TranslationState,
} from "../../../yap-frontend-rs/pkg";

// Captures use serde JSON; only reducer inputs need a bridge representation.
function parseFixture(json: string): Fixture {
  const parsed = JSON.parse(json) as Fixture;
  if (parsed.screen !== "Review" || parsed.view.step.type !== "Challenge")
    return parsed;
  const fixture = parsed.view.step.view as unknown as {
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
    view: {
      ...parsed.view,
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
    },
  };
}

const log = (...args: unknown[]) => console.log("fixture action", ...args);
const rejectCompletion = (...args: unknown[]) => { log(...args); return false; };

export function FixturePage() {
  const { name } = useParams();
  const { accessToken, userInfo } = useOutletContext<AppContextType>();
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
        if (!response.ok)
          throw new Error(`Fixture: ${response.status}`);
        const fixture = parseFixture(await response.text());
        if (!abort.signal.aborted) setLoaded({ name: name!, fixture });
      })
      .catch((error) => {
        if (!abort.signal.aborted) setError({ name, message: String(error) });
      });
    return () => abort.abort();
  }, [name]);
  if (error?.name === name) return <p>{error?.message}</p>;
  if (deckState.view.phase.type !== "Ready")
    return <DeckLoadStatus phase={deckState.view.phase} retry={deckState.retry} />;
  if (
    loaded?.name !== name ||
    !loaded ||
    !deckState.deck
  )
    return <p>Loading fixture…</p>;
  const { deck } = deckState;
  const snapshot = loaded.fixture;
  if (snapshot.screen !== "Review") {
    const props = { deck, userInfo };
    return (
      <div data-fixture-rendered={name} inert>
        {snapshot.screen === "Home" && (
          <HomeScreen {...props} view={snapshot.view} />
        )}
        {snapshot.screen === "Stats" && (
          <StatsScreen {...props} view={snapshot.view} />
        )}
        {snapshot.screen === "Goals" && (
          <GoalsScreen {...props} view={snapshot.view} />
        )}
        {snapshot.screen === "Due" && (
          <DueWordsScreen {...props} view={snapshot.view} />
        )}
      </div>
    );
  }
  const fixture = snapshot.view;
  return (
    // Mirror ReviewPage's shell so captures show the same header and progress
    // bar the real screen does — iOS renders the whole app, so a bare fixture
    // would make every web/iOS pair differ by the chrome alone. The signup nag
    // stays off: it varies with auth state and would make captures unstable.
    <TopPageLayout
      userInfo={userInfo}
      headerProps={{
        backButton: { label: "Home", onBack: log },
        showSignupNag: false,
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
            pendingReviewScope: "fixture",
            setPlacement: log,
            onRating: rejectCompletion,
            onTranslationComplete: rejectCompletion,
            onTranscriptionComplete: rejectCompletion,
            onCantListen: log,
            onCantSpeak: log,
            addEvent: log,
            undoRestrictions: log,
            setSentenceList: log,
            commitSentenceList: log,
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
