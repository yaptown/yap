import type { ReviewHost, ReviewActions } from "./ReviewActions";
import type { ReviewScreenView } from "../../../yap-frontend-rs/pkg";
import { ChallengeView } from "./challenges/ChallengeView";
import { IdleScreen } from "./IdleScreen";
import { AccomplishmentScreen } from "./ladder/AccomplishmentScreen";
import { ReviewPlanScreen } from "./ladder/ReviewPlanScreen";
import { PlacementTest } from "./ladder/PlacementTest";
import { SetDisplayName } from "./ladder/SetDisplayName";

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
              <ReviewPlanScreen
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
              <IdleScreen
                view={step.view}
                deck={host.deck}
                addEvent={actions.addEvent}
                undoRestrictions={actions.undoRestrictions}
                setSentenceList={actions.setSentenceList}
                commitSentenceList={actions.commitSentenceList}
                showEngagementPrompts={view.offer_engagement}
              />
            );
          case "Challenge":
            return (
              <ChallengeView
                {...host}
                pendingReviewScope={actions.pendingReviewScope}
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
