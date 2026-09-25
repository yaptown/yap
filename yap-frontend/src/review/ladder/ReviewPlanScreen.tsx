import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import type {
  DeckEvent,
  ReviewPlanView,
} from "../../../../yap-frontend-rs/pkg";
import { TargetLanguageText } from "../../components/TargetLanguageText";
import { WeekProgressStrip } from "../WeekProgressStrip";

/// Shared layout for committing to a set of reviews: the lockup offer
/// ("Today's review plan") and releasing more cards from lockup. Rust groups
/// the cards by type; this shows the groups above a single commit button.
export function ReviewPlanCard({
  plan,
  onCommit,
  showWeek = true,
}: {
  plan: ReviewPlanView;
  onCommit: () => void;
  showWeek?: boolean;
}) {
  return (
    <div className="flex flex-col flex-1 gap-4">
      <Card className="max-w w-full p-6 gap-6 select-none" animate>
        <h2 className="text-xl font-semibold text-center">{plan.title}</h2>

        {plan.groups.map((group) => (
          <div key={group.heading} className="space-y-2">
            <p className="text-sm font-medium text-muted-foreground text-center">
              {group.heading}
            </p>
            <div className="flex flex-wrap justify-center gap-y-1">
              {group.cards.map((card, i) => (
                <span
                  key={i}
                  className={`px-3 text-sm font-medium ${i > 0 ? "border-l border-border" : ""}`}
                >
                  <TargetLanguageText language={plan.target_language}>
                    {card}
                  </TargetLanguageText>
                </span>
              ))}
            </div>
          </div>
        ))}

        <Button onClick={onCommit} size="lg" className="w-full">
          {plan.accept_label}
        </Button>
      </Card>

      {showWeek && <WeekProgressStrip week={plan.week} className="mt-auto mb-2" />}
    </div>
  );
}

/// Session-start screen shown when a review backlog builds up: keeps the most
/// due cards active and sets the rest aside ("lockup").
export function ReviewPlanScreen({
  offer,
  onAccept,
}: {
  offer: ReviewPlanView;
  onAccept: (event: DeckEvent) => void;
}) {
  return <ReviewPlanCard plan={offer} onCommit={() => onAccept(offer.event)} />;
}
