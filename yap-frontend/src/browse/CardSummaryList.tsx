import { useState } from "react";
import TimeAgo from "react-timeago";
import type { CardSummary, Language } from "../../../yap-frontend-rs/pkg";
import { Card } from "../components/ui/card";
import { Badge } from "../components/ui/badge";
import { TargetLanguageText } from "../components/TargetLanguageText";

export function CardSummaryList({
  cards,
  targetLanguage,
  timestampMs,
}: {
  cards: CardSummary[];
  targetLanguage: Language;
  timestampMs: number;
}) {
  const [revealed, setRevealed] = useState<Set<string>>(() => new Set());
  return (
    <Card className="overflow-hidden p-0 gap-0">
      <table className="w-full table-fixed">
        <thead>
          <tr className="border-b bg-muted/50">
            <th className="text-left p-3 font-medium">Word</th>
            <th className="text-left p-3 font-medium">Ready</th>
          </tr>
        </thead>
        <tbody>
          {cards.map((card) => {
            const key = JSON.stringify(card.card_indicator);
            const listening = card.card_indicator.type === "ListeningGram";
            return (
              <tr key={key} className="border-b last:border-b-0">
                <td className="p-3 break-words">
                  <div className="flex flex-col items-start gap-1">
                    {listening && !revealed.has(key) ? (
                      <button
                        type="button"
                        className="flex flex-wrap items-center gap-2 text-left font-medium focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                        aria-label="Reveal listening lexeme"
                        onClick={() =>
                          setRevealed((previous) => new Set(previous).add(key))
                        }
                      >
                        <span className="select-none blur-sm">
                          <TargetLanguageText language={targetLanguage}>
                            {card.card_text}
                          </TargetLanguageText>
                        </span>
                        <span className="text-xs italic text-muted-foreground">
                          Tap to reveal
                        </span>
                      </button>
                    ) : (
                      <span className="font-medium">
                        <TargetLanguageText language={targetLanguage}>
                          {card.card_text}
                        </TargetLanguageText>
                      </span>
                    )}
                    {card.card_subtitle && (
                      <span className="text-sm text-muted-foreground">
                        {card.card_subtitle}
                      </span>
                    )}
                  </div>
                </td>
                <td className="p-3 text-sm text-muted-foreground">
                  {card.due_timestamp_ms <= timestampMs ? (
                    <Badge variant="outline">Ready now</Badge>
                  ) : (
                    <TimeAgo date={new Date(card.due_timestamp_ms)} />
                  )}
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </Card>
  );
}
