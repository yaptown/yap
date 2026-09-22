import { Button } from "@/components/ui/button.tsx";
import { Card } from "@/components/ui/card";
import { ArrowLeft, TriangleAlert } from "lucide-react";
import type {
  Deck,
  Language,
  PlacementSession,
} from "../../../yap-frontend-rs/pkg";
import { Progress } from "@/components/ui/progress";
import { TargetLanguageText } from "./TargetLanguageText";

import { get_placement_session_info, toggle_placement_word } from "../../../yap-frontend-rs/pkg";

interface PlacementTestProps {
  deck: Deck;
  targetLanguage: Language;
  session: PlacementSession;
  setSession: (session: PlacementSession) => void;
  onComplete: (session: PlacementSession) => void;
}

export function PlacementTest({
  deck,
  targetLanguage,
  onComplete,
  session,
  setSession,
}: PlacementTestProps) {
  const info = get_placement_session_info(session);
  const { round, words } = session;
  const selectedWords = new Set(session.selected_words);
  const tooAdvanced = info.too_advanced;

  const toggleWord = (word: string) => setSession(toggle_placement_word(session, word));
  const handleNext = () => setSession(deck.advance_placement_session(session));
  const handleBack = () => setSession(deck.start_placement_session());

  return (
    <Card className="max-w w-full p-0 gap-0 select-none overflow-hidden" animate>
      <Progress
        value={info.progress_percent}
        className="rounded-none"
      />
      <div className="p-6 space-y-4">
        {info.finished ? (
          (() => {
            return (
              <>
                <div className="space-y-2">
                  <h2 className="text-xl font-semibold flex items-center gap-2">
                    {tooAdvanced && (
                      <TriangleAlert className="w-5 h-5 text-caution" />
                    )}
                    {tooAdvanced
                      ? "You might be too advanced"
                      : "Ready to Start!"}
                  </h2>
                </div>
                <p className="text-muted-foreground">
                  {tooAdvanced
                    ? "Yap.Town is designed for intermediate learners. We'll still try our best to find words you don't know!"
                    : "We've analyzed your knowledge level and will tailor your learning experience."}
                </p>
                <Button
                  onClick={() => onComplete(session)}
                  size="lg"
                  className="w-full"
                >
                  {tooAdvanced ? "Continue Anyway" : "Begin Learning"}
                </Button>
              </>
            );
          })()
        ) : (
          <>
            <div className="space-y-2">
              <div className="flex items-center justify-between mb-2">
                <div className="flex items-center gap-2">
                  {info.can_restart && (
                    <Button
                      variant="ghost"
                      size="icon"
                      onClick={handleBack}
                      className="h-8 w-8"
                      title="Go back"
                    >
                      <ArrowLeft className="w-5 h-5" />
                    </Button>
                  )}
                  <h2 className="text-xl font-semibold">Placement Test</h2>
                </div>
                <span className="text-sm text-muted-foreground">
                  Round {round} of {info.total_rounds}
                </span>
              </div>
              <p className="text-muted-foreground">
                Tap the words you know, then press Next
              </p>
            </div>

            <div className="grid grid-cols-2 sm:grid-cols-3 gap-3 mb-6">
              {words.map((pw) => {
                const isSelected = selectedWords.has(pw.word);
                return (
                  <button
                    key={pw.word}
                    onClick={() => toggleWord(pw.word)}
                    className={`
                  p-4 rounded-lg border-2 transition-all
                  ${
                    isSelected
                      ? "border-primary bg-primary/10 scale-95"
                      : "border-border hover:border-primary/50 hover:bg-accent"
                  }
                `}
                  >
                    {isSelected ? (
                      <span
                        key="def"
                        className="text-lg text-muted-foreground truncate block placement-reveal"
                      >
                        {pw.definition}
                      </span>
                    ) : (
                      <span key="word" className="text-lg font-medium">
                        <TargetLanguageText language={targetLanguage}>
                          {pw.word}
                        </TargetLanguageText>
                      </span>
                    )}
                  </button>
                );
              })}
            </div>

            <Button onClick={handleNext} className="w-full" size="lg">
              Next
            </Button>
          </>
        )}
      </div>
    </Card>
  );
}
