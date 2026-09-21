import type { ReactNode } from "react";
import type { SentenceList } from "@/hooks/useSentenceList";
import type { ReviewScreenView, Deck, DeckEvent, PlacementSession, Rating, PartGraded, LiteralGrades, Gram, Heteronym } from "../../../yap-frontend-rs/pkg";
import { ChallengeView } from "./challenges/ChallengeView";
import { NoCardsReady } from "./no-cards-ready";
import { AccomplishmentScreen } from "./AccomplishmentScreen";
import { LockupOfferScreen } from "./LockupOffer";
import { PlacementTest } from "./PlacementTest";
import { SetDisplayName } from "./SetDisplayName";

export type ReviewHost = {
  deck: Deck;
  accessToken: string | undefined;
  autoplayed: boolean;
  setAutoplayed: () => void;
  menuExtras?: ReactNode;
};
export type ReviewActions = {
  onRating: (rating: Rating) => void;
  onTranslationComplete: (
    grade: { literalGrades: LiteralGrades; phrasesRemembered: Gram<string>[]; phrasesForgot: Gram<string>[] } | { perfect: string | null },
    tapped: Heteronym<string>[], submission: string, completedAtMs: number,
  ) => void;
  onTranscriptionComplete: (grade: PartGraded[], completedAtMs: number) => void;
  onCantListen: () => void;
  onCantSpeak: () => void;
  addEvent: (event: DeckEvent) => void;
  undoRestrictions: () => void;
  setSentenceList: (list: SentenceList) => void;
  dismissAccomplishment: () => void;
  setPlacement: (session: PlacementSession) => void;
  completePlacementTest: (session: PlacementSession) => void;
  saveDisplayName: (name: string) => Promise<void> | void;
  skipDisplayName: () => void;
};

/** Live and captured reviews share this renderer; Rust alone selects the step. */
export function ReviewScreen({
  view,
  host,
  actions,
}: {
  view: ReviewScreenView;
  host: ReviewHost;
  actions: ReviewActions;
}) {
  const step = view.step;
  return (
    <div className="flex flex-col flex-1 gap-2">
      {(() => {
        switch (step.type) {
          case "PlacementTest":
            return (
              <PlacementTest
                deck={host.deck}
                targetLanguage={view.target_language}
                session={step.view}
                setSession={actions.setPlacement}
                onComplete={actions.completePlacementTest}
              />
            );
          case "ReviewPlan":
            return (
              <LockupOfferScreen
                offer={step.view}
                onAccept={actions.addEvent}
              />
            );
          case "SetDisplayName":
            return (
              <SetDisplayName
                onSave={actions.saveDisplayName}
                totalReviewsCompleted={BigInt(view.total_reviews)}
                onSkip={actions.skipDisplayName}
              />
            );
          case "Accomplishment":
            return (
              <AccomplishmentScreen
                view={step.view}
                addEvent={actions.addEvent}
                onDismiss={actions.dismissAccomplishment}
              />
            );
          case "Idle":
            return (
              <NoCardsReady
                view={step.view}
                deck={host.deck}
                addEvent={actions.addEvent}
                undoRestrictions={actions.undoRestrictions}
                setSentenceList={actions.setSentenceList}
                showEngagementPrompts={view.offer_engagement}
              />
            );
          case "Challenge":
            return (
              <ChallengeView
                {...host}
                onRating={actions.onRating}
                onTranslationComplete={actions.onTranslationComplete}
                onTranscriptionComplete={actions.onTranscriptionComplete}
                onCantListen={actions.onCantListen}
                onCantSpeak={actions.onCantSpeak}
                nativeLanguage={view.native_language}
                challenge={step.view.challenge}
                initialState={step.view.transcription ?? undefined}
                translationState={step.view.translation ?? undefined}
                targetLanguage={view.target_language}
                totalReviewsCompleted={BigInt(view.total_reviews)}
                totalCount={Number(view.total_count)}
              />
            );
        }
      })()}
    </div>
  );
}
