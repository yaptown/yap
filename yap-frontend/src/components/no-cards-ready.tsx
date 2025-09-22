import { Button } from "@/components/ui/button";
import TimeAgo from "react-timeago";
import { EngagementPrompts } from "@/components/engagement-prompts";
import type {
  AddCardOptions,
  CardSummary,
  CardType,
  Deck,
  Language,
} from "../../../yap-frontend-rs/pkg";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { ChevronDown, AlertCircle } from "lucide-react";
import { AnimatedCard } from "./AnimatedCard";
import { Alert, AlertDescription } from "@/components/ui/alert";

interface NoCardsReadyProps {
  nextDueCard: CardSummary | null;
  showEngagementPrompts: boolean;
  addNextCards: (card_type: CardType | undefined, count: number) => void;
  addCardOptions: AddCardOptions;
  targetLanguage: Language;
  deck: Deck;
}

export function NoCardsReady({
  nextDueCard,
  showEngagementPrompts,
  addNextCards,
  addCardOptions,
  targetLanguage,
  deck,
}: NoCardsReadyProps) {
  const numCanAddTargetLanguage =
    addCardOptions.manual_add.find(
      ([, card_type]) => card_type === "TargetLanguage"
    )?.[0] || 0;
  const numCanAddListening =
    addCardOptions.manual_add.find(
      ([, card_type]) => card_type === "Listening"
    )?.[0] || 0;
  const numCanAddLetterPronunciation =
    addCardOptions.manual_add.find(
      ([, card_type]) => card_type === "LetterPronunciation"
    )?.[0] || 0;
  const numCanSmartAdd = addCardOptions.smart_add;

  // Calculate if workload looks light
  const pastWeekAverage = deck.get_past_week_challenge_average();
  const upcomingStats = deck.get_upcoming_week_review_stats();
  const cardsAddedPast16Hours = deck.get_cards_added_in_past_hours(16);
  const showLightWorkloadNotification =
    cardsAddedPast16Hours < 20 &&
    (upcomingStats.total_reviews < pastWeekAverage * 7 ||
      upcomingStats.max_per_day < 10) && // Less upcoming reviews than past week average
    upcomingStats.max_per_day <= 50 && // No single day has more than 50 reviews
    (numCanSmartAdd > 0 ||
      numCanAddTargetLanguage > 0 ||
      numCanAddListening > 0 ||
      numCanAddLetterPronunciation > 0) && // Can add cards
    deck.num_cards() > 40; // has used yap a bit

  const add_cards: [number, CardType | undefined][] = [];
  if (numCanSmartAdd > 0) {
    add_cards.push([numCanSmartAdd, undefined]);
  }
  if (numCanAddTargetLanguage > 0) {
    add_cards.push([numCanAddTargetLanguage, "TargetLanguage"]);
  }
  if (numCanAddListening > 0) {
    add_cards.push([numCanAddListening, "Listening"]);
  }
  if (numCanAddLetterPronunciation > 0) {
    add_cards.push([numCanAddLetterPronunciation, "LetterPronunciation"]);
  }

  const targetLanguageSpan = (
    <span style={{ fontWeight: "bold" }}>{targetLanguage} → English</span>
  );
  const listeningSpan = (
    <span style={{ fontWeight: "bold" }}>{targetLanguage} listening</span>
  );
  const pronunciationSpan = (
    <span style={{ fontWeight: "bold" }}>{targetLanguage} pronunciation</span>
  );

  return (
    <div className="space-y-4">
      <AnimatedCard className="bg-card text-card-foreground rounded-lg p-6 gap-6 flex flex-col text-center border">
        {showLightWorkloadNotification && (
          <Alert>
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>
              Your upcoming workload looks a little light. Consider adding new
              cards to maintain your learning momentum!
            </AlertDescription>
          </Alert>
        )}
        <div className="flex flex-col gap-2">
          <p className="text-lg">No cards ready for review!</p>
          <p className="text-muted-foreground">
            Great job! Your next review is{" "}
            {nextDueCard ? (
              <TimeAgo date={new Date(nextDueCard.due_timestamp_ms)} />
            ) : (
              "soon"
            )}
            .
          </p>
        </div>

        <div className="space-y-4">
          {add_cards.length > 0 ? (
            <div className="flex justify-center">
              <Button
                onClick={() => addNextCards(add_cards[0][1], add_cards[0][0])}
                variant="default"
                className={add_cards.length > 1 ? "rounded-r-none" : ""}
              >
                Add {add_cards[0][0]} new{" "}
                {add_cards[0][1] === undefined
                  ? ""
                  : add_cards[0][1] === "TargetLanguage"
                  ? targetLanguageSpan
                  : add_cards[0][1] === "Listening"
                  ? listeningSpan
                  : pronunciationSpan}{" "}
                cards to my deck
              </Button>
              {add_cards.length > 1 && (
                <DropdownMenu>
                  <DropdownMenuTrigger asChild>
                    <Button
                      variant="default"
                      className="rounded-l-none border-l border-l-primary-foreground/20 px-2"
                    >
                      <ChevronDown className="h-4 w-4" />
                    </Button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align="end">
                    {add_cards.slice(1).map(([count, card_type]) => (
                      <DropdownMenuItem
                        key={card_type || "smart"}
                        onClick={() => addNextCards(card_type, count)}
                      >
                        Add {count}{" "}
                        {card_type === "TargetLanguage"
                          ? targetLanguageSpan
                          : card_type === "Listening"
                          ? listeningSpan
                          : card_type === "LetterPronunciation"
                          ? pronunciationSpan
                          : ""}{" "}
                        cards
                      </DropdownMenuItem>
                    ))}
                  </DropdownMenuContent>
                </DropdownMenu>
              )}
            </div>
          ) : (
            <p className="text-muted-foreground">
              You've learned all available words! Keep practicing to master
              them.
            </p>
          )}
        </div>
      </AnimatedCard>

      {showEngagementPrompts && (
        <EngagementPrompts language={targetLanguage} />
      )}
    </div>
  );
}
