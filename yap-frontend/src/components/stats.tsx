import { useState, lazy, Suspense } from "react";
import { Badge } from "@/components/ui/badge";
import TimeAgo from "react-timeago";
import type { Deck } from "../../../yap-frontend-rs/pkg";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import { ChevronDown, ChevronRight } from "lucide-react";

// Lazy load the chart component - only loads when needed
const FrequencyKnowledgeChart = lazy(() => 
  import('./FrequencyKnowledgeChart').then(module => ({
    default: module.FrequencyKnowledgeChart
  }))
);

interface StatsProps {
  deck: Deck;
}

export function Stats({ deck }: StatsProps) {
  const reviewInfo = deck.get_review_info([]);
  const allCardsSummary = deck.get_all_cards_summary();

  const now = Date.now();
  const readyCards = allCardsSummary.filter(
    (card) => card.due_timestamp_ms <= now,
  );
  const notReadyCards = allCardsSummary.filter(
    (card) => card.due_timestamp_ms > now,
  );

  const [visibleCount, setVisibleCount] = useState(10);
  const [nextBatchSize, setNextBatchSize] = useState(10);
  const visibleCards = [...readyCards, ...notReadyCards.slice(0, visibleCount)];
  
  const [isGraphsOpen, setIsGraphsOpen] = useState(false);

  return (
    <div className="mt-4">
      <div className="mb-4">
        <h2 className="text-2xl font-semibold">Stats</h2>
        <div className="grid grid-cols-1 md:grid-cols-1 gap-4 mt-3">
          <div className="bg-card border rounded-lg p-4">
            <p className="text-sm text-muted-foreground mb-1">XP</p>
            <p className="text-2xl font-bold">{deck.get_xp().toFixed(2)}</p>
            <p className="text-sm text-muted-foreground mt-1">
              total stability gained
            </p>
          </div>
        </div>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mt-3">
          <div className="bg-card border rounded-lg p-4">
            <p className="text-sm text-muted-foreground mb-1">Total Cards</p>
            <p className="text-2xl font-bold">{allCardsSummary.length}</p>
            <p className="text-sm text-muted-foreground mt-1">
              {reviewInfo.due_count || 0} ready now
            </p>
          </div>
          <div className="bg-card border rounded-lg p-4">
            <p className="text-sm text-muted-foreground mb-1">Words Known</p>
            <p className="text-2xl font-bold">
              {(deck.get_percent_of_words_known() * 100).toFixed(2)}%
            </p>
            <p className="text-sm text-muted-foreground mt-1">of total</p>
          </div>
          <div className="bg-card border rounded-lg p-4">
            <p className="text-sm text-muted-foreground mb-1">Daily Streak</p>
            <p className="text-2xl font-bold">{deck.get_daily_streak()}</p>
            <p className="text-sm text-muted-foreground mt-1">days</p>
          </div>
          <div className="bg-card border rounded-lg p-4">
            <p className="text-sm text-muted-foreground mb-1">Total Reviews</p>
            <p className="text-2xl font-bold">{deck.get_total_reviews()}</p>
            <p className="text-sm text-muted-foreground mt-1">all time</p>
          </div>
        </div>
      </div>
      <div className="bg-card border rounded-lg overflow-hidden">
        <table className="w-full table-fixed">
          <thead>
            <tr className="border-b bg-muted/50">
              <th className="text-left p-3 font-medium w-1/4">Word</th>
              <th className="text-left p-3 font-medium w-1/4">State</th>
              <th className="text-left p-3 font-medium w-1/2">Ready</th>
            </tr>
          </thead>
          <tbody>
            {visibleCards.map((card, index) => {
              let shortDescription = "";
              let pos = "";
              
              if ("TargetLanguage" in card.card_indicator) {
                if ("Heteronym" in card.card_indicator.TargetLanguage.lexeme) {
                  shortDescription = card.card_indicator.TargetLanguage.lexeme.Heteronym.word;
                  pos = card.card_indicator.TargetLanguage.lexeme.Heteronym.pos;
                } else {
                  shortDescription = card.card_indicator.TargetLanguage.lexeme.Multiword;
                }
              } else {
                shortDescription = `/${card.card_indicator.ListeningHomophonous.pronunciation}/`;
              }

              const isReady = card.due_timestamp_ms <= now;
              return (
                <tr
                  key={index}
                  className={`border-b ${isReady ? "bg-green-500/10" : ""}`}
                >
                  <td className="p-3 font-medium">
                    {shortDescription}
                    {pos && (
                      <span className="ml-2 text-muted-foreground text-sm">
                        ({pos.toLowerCase()})
                      </span>
                    )}
                  </td>
                  <td className="p-3">
                    <Badge variant="outline">{card.state}</Badge>
                  </td>
                  <td className="p-3 text-sm text-muted-foreground">
                    {isReady ? (
                      <span className="text-green-500 font-medium">
                        Ready now
                      </span>
                    ) : (
                      <TimeAgo date={new Date(card.due_timestamp_ms)} />
                    )}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
        {notReadyCards.length > visibleCount && (
          <div className="border-t">
            <button
              onClick={() => {
                setVisibleCount((c) => c + nextBatchSize);
                setNextBatchSize((s) => s * 10);
              }}
              className="w-full py-3 text-sm text-muted-foreground hover:text-foreground hover:bg-muted/50 transition-colors duration-200 font-medium"
            >
              Show {Math.min(nextBatchSize, notReadyCards.length - visibleCount)} more cards
            </button>
          </div>
        )}
      </div>
      
      {/* Collapsible Graphs Section */}
      <Collapsible open={isGraphsOpen} onOpenChange={setIsGraphsOpen} className="mt-6">
        <CollapsibleTrigger className="flex items-center gap-2 text-lg font-semibold hover:text-muted-foreground transition-colors">
          {isGraphsOpen ? <ChevronDown className="h-5 w-5" /> : <ChevronRight className="h-5 w-5" />}
          Graphs
        </CollapsibleTrigger>
        <CollapsibleContent className="mt-4">
          <div className="bg-card border rounded-lg p-6">
            <h3 className="text-base font-semibold mb-4">Predicted Knowledge by Word Frequency</h3>
            <p className="text-sm text-muted-foreground mb-4">
              This chart shows how well you're predicted to know words based on their frequency in the language.
            </p>
            <Suspense fallback={
              <div className="h-[400px] flex items-center justify-center text-muted-foreground">
                <p>Loading chart...</p>
              </div>
            }>
              <FrequencyKnowledgeChart deck={deck} />
            </Suspense>
          </div>
        </CollapsibleContent>
      </Collapsible>
    </div>
  );
}
