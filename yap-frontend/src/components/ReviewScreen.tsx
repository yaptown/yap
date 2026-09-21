import type { ComponentProps } from "react";
import type { ReviewScreenView } from "../../../yap-frontend-rs/pkg";
import { ChallengeView } from "./challenges/ChallengeView";
import { NoCardsReady } from "./no-cards-ready";
import { AccomplishmentScreen } from "./AccomplishmentScreen";
import { LockupOfferScreen } from "./LockupOffer";
import { PlacementTest } from "./PlacementTest";
import { SetDisplayName } from "./SetDisplayName";

type ChallengeActions = Omit<
  ComponentProps<typeof ChallengeView>,
  | "challenge"
  | "initialState"
  | "translationState"
  | "targetLanguage"
  | "totalReviewsCompleted"
  | "totalCount"
>;
export type ReviewActions = ChallengeActions & {
  addEvent: ComponentProps<typeof NoCardsReady>["addEvent"];
  undoRestrictions: () => void;
  setSentenceList: ComponentProps<typeof NoCardsReady>["setSentenceList"];
  dismissAccomplishment: () => void;
  completePlacementTest: ComponentProps<typeof PlacementTest>["onComplete"];
  saveDisplayName: ComponentProps<typeof SetDisplayName>["onSave"];
  completeDisplayName: () => void;
  skipDisplayName: () => void;
};

/** Live and captured reviews share this renderer; Rust alone selects the step. */
export function ReviewScreen({
  view,
  actions,
}: {
  view: ReviewScreenView;
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
                deck={actions.deck}
                targetLanguage={view.target_language}
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
                onComplete={actions.completeDisplayName}
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
                deck={actions.deck}
                addEvent={actions.addEvent}
                undoRestrictions={actions.undoRestrictions}
                setSentenceList={actions.setSentenceList}
                showEngagementPrompts={view.offer_engagement}
              />
            );
          case "Challenge":
            return (
              <ChallengeView
                {...actions}
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
