import { Button } from "@/components/ui/button"
import TimeAgo from 'react-timeago'
import { EngagementPrompts } from '@/components/engagement-prompts'
import type { AddCardOptions, CardSummary, CardType, Language } from '../../../yap-frontend-rs/pkg'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { ChevronDown } from "lucide-react"
import { AnimatedCard } from "./AnimatedCard"

interface NoCardsDueProps {
  nextDueCard: CardSummary | null
  showEngagementPrompts: boolean
  addNextCards: (card_type: CardType | undefined, count: number) => void
  addCardOptions: AddCardOptions
  targetLanguage: Language
}

export function NoCardsDue({ nextDueCard, showEngagementPrompts, addNextCards, addCardOptions, targetLanguage }: NoCardsDueProps) {
  const numCanAddTargetLanguage = addCardOptions.manual_add.find(([, card_type]) => card_type === 'TargetLanguage')?.[0] || 0
  const numCanAddListening = addCardOptions.manual_add.find(([, card_type]) => card_type === 'Listening')?.[0] || 0
  const numCanSmartAdd = addCardOptions.smart_add

  const add_cards: [number, CardType | undefined][] = []
  if (numCanSmartAdd > 0) {
    add_cards.push([numCanSmartAdd, undefined])
  }
  if (numCanAddTargetLanguage > 0) {
    add_cards.push([numCanAddTargetLanguage, "TargetLanguage"])
  }
  if (numCanAddListening > 0) {
    add_cards.push([numCanAddListening, "Listening"])
  }

  const targetLanguageSpan = <span style={{ fontWeight: "bold" }}>{targetLanguage} → English</span>
  const listeningSpan = <span style={{ fontWeight: "bold" }}>{targetLanguage} listening</span>

  return (
    <div className="space-y-4">
      <AnimatedCard className="bg-card text-card-foreground rounded-lg p-12 gap-6 flex flex-col text-center border">
        <div className="flex flex-col gap-2">
          <p className="text-lg">No cards due for review!</p>
          <p className="text-muted-foreground">
            Great job! Your next review is due {nextDueCard ? <TimeAgo date={new Date(nextDueCard.due_timestamp_ms)} /> : 'soon'}.
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
                Add {add_cards[0][0]} new {add_cards[0][1] === undefined ? "" : add_cards[0][1] === "TargetLanguage" ? targetLanguageSpan : listeningSpan} cards to my deck
              </Button>
              {(add_cards.length > 1) && (
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
                      <DropdownMenuItem onClick={() => addNextCards(card_type, count)}>
                        Add {count} {card_type === "TargetLanguage" ? targetLanguageSpan : listeningSpan} cards
                      </DropdownMenuItem>
                    ))}
                  </DropdownMenuContent>
                </DropdownMenu>
              )}
            </div>
          ) : (
            <p className="text-muted-foreground">
              You've learned all available words! Keep practicing to master them.
            </p>
          )}
        </div>
      </AnimatedCard>

      {showEngagementPrompts && <EngagementPrompts />}
    </div>
  )
}
