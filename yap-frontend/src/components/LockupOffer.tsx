import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import type {
  CardSummary,
  DayProgress,
  DeckEvent,
  Language,
  ReviewPlanView,
} from "../../../yap-frontend-rs/pkg";
import { TargetLanguageText } from "./TargetLanguageText";
import { WeekProgressStrip } from "./WeekProgressStrip";

const CARD_GROUPS = [
  { type: "WrittenGram", label: "reading" },
  { type: "ListeningGram", label: "listening" },
  { type: "LetterPronunciation", label: "pronunciation" },
] as const;

interface ReviewPlanCardProps {
  title: string;
  cards: CardSummary[];
  buttonLabel: string;
  onCommit: () => void;
  week: DayProgress[];
  targetLanguage: Language;
}

/// Shared layout for committing to a set of reviews: the lockup offer
/// ("Today's review plan") and releasing more cards from lockup. Shows the
/// cards grouped by type above a single commit button.
export function ReviewPlanCard({
  title,
  cards,
  buttonLabel,
  onCommit,
  week,
  targetLanguage,
}: ReviewPlanCardProps) {
  const byType = new Map<string, CardSummary[]>();
  for (const card of cards) {
    const type = card.card_indicator.type;
    byType.set(type, [...(byType.get(type) ?? []), card]);
  }

  return (
    <div className="flex flex-col flex-1 gap-4">
      <Card className="max-w w-full p-6 gap-6 select-none" animate>
        <h2 className="text-xl font-semibold text-center">{title}</h2>

        {CARD_GROUPS.map(({ type, label }) => {
          const group = byType.get(type);
          if (!group || group.length === 0) return null;
          return (
            <div key={type} className="space-y-2">
              <p className="text-sm font-medium text-muted-foreground text-center">
                {group.length} {label} {group.length === 1 ? "card" : "cards"}
              </p>
              <div className="flex flex-wrap justify-center gap-y-1">
                {group.map((card, i) => (
                  <span
                    key={i}
                    className={`px-3 text-sm font-medium ${i > 0 ? "border-l border-border" : ""}`}
                  >
                    <TargetLanguageText language={targetLanguage}>
                      {card.card_text}
                    </TargetLanguageText>
                  </span>
                ))}
              </div>
            </div>
          );
        })}

        <Button onClick={onCommit} size="lg" className="w-full">
          {buttonLabel}
        </Button>
      </Card>

      <WeekProgressStrip week={week} className="mt-auto mb-2" />
    </div>
  );
}

interface LockupOfferScreenProps {
  offer: ReviewPlanView;
  onAccept: (event: DeckEvent) => void;
}

/// Session-start screen shown when a review backlog builds up: keeps the most
/// due cards active and sets the rest aside ("lockup").
export function LockupOfferScreen({
  offer,
  onAccept,
}: LockupOfferScreenProps) {
  const preview = offer.cards;
  const lockEvent = offer.event;

  return (
    <ReviewPlanCard
      title="Today's review plan:"
      cards={preview}
      buttonLabel="Let's go!"
      onCommit={() => onAccept(lockEvent)}
      week={offer.week}
      targetLanguage={offer.target_language}
    />
  );
}
