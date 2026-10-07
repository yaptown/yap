import { t } from "@/lib/i18n";
import { Button } from "@/components/ui/button.tsx";
import { Card } from "@/components/ui/card";
import { ArrowLeft, TriangleAlert } from "lucide-react";
import type {
  Deck,
  Language,
  PlacementSession,
} from "../../../../yap-frontend-rs/pkg";
import { Progress } from "@/components/ui/progress";
import { TargetLanguageText } from "../../components/TargetLanguageText";

import {
  get_placement_session_info,
  toggle_placement_word,
} from "../../../../yap-frontend-rs/pkg";

interface PlacementTestProps {
  deck: Deck;
  targetLanguage: Language;
  session: PlacementSession;
  setSession: (session: PlacementSession) => void;
  onComplete: (session: PlacementSession) => void;
  /** Label of the finishing button; defaults to "Begin Learning". */
  completeLabel?: string;
}

export function PlacementTest({
  deck,
  targetLanguage,
  onComplete,
  session,
  setSession,
  completeLabel,
}: PlacementTestProps) {
  const info = get_placement_session_info(session);
  const { round, words } = session;
  const selectedWords = new Set(session.selected_words);
  const tooAdvanced = info.too_advanced;

  const toggleWord = (word: string) =>
    setSession(toggle_placement_word(session, word));
  const handleNext = () => setSession(deck.advance_placement_session(session));
  const handleBack = () => setSession(deck.start_placement_session());

  return (
    <Card
      className="max-w w-full p-0 gap-0 select-none overflow-hidden"
      animate
    >
      <Progress value={info.progress_percent} className="rounded-none" />
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
                      ? t("home-you-might-be-too-advanced")
                      : t("home-ready-to-start")}
                  </h2>
                </div>
                <p className="text-muted-foreground">
                  {tooAdvanced
                    ? t(
                        "placement-test-view-yap-town-is-designed-for-intermediate-learners-we-ll",
                      )
                    : t(
                        "placement-test-view-we-ve-analyzed-your-knowledge-level-and-will-tailor",
                      )}
                </p>
                <Button
                  onClick={() => onComplete(session)}
                  size="lg"
                  className="w-full"
                >
                  {tooAdvanced
                    ? t("placement-test-continue-anyway")
                    : (completeLabel ?? t("placement-test-begin-learning"))}
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
                      title={t("placement-test-go-back")}
                    >
                      <ArrowLeft className="w-5 h-5" />
                    </Button>
                  )}
                  <h2 className="text-xl font-semibold">
                    {t("home-placement-test")}
                  </h2>
                </div>
                <span className="text-sm text-muted-foreground">
                  {t("placement-round", {
                    round: round,
                    total: info.total_rounds,
                  })}
                </span>
              </div>
              <p className="text-muted-foreground">
                {t("placement-test-tap-the-words-you-know-then-press-next")}
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
              {t("common-next")}
            </Button>
          </>
        )}
      </div>
    </Card>
  );
}
