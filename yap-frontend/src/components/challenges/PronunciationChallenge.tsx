import type {
  PronunciationCue,
  Language,
  Rating,
} from "../../../../yap-frontend-rs/pkg";
import Markdown from "react-markdown";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { useCallback, useEffect, useState } from "react";
import { AudioButton } from "../AudioButton";
import { CantSpeakButton } from "../CantSpeakButton";
import { AudioErrorBanner } from "../AudioErrorBanner";
import { useBackground } from "../background-context";
import { PlayfulArrow } from "../PlayfulArrow";
import { match } from "ts-pattern";
import { ArrowLeft, ArrowRight } from "lucide-react";
import { TargetLanguageText } from "../TargetLanguageText";

interface PronunciationChallengeProps {
  pattern: string;
  guide: {
    position: "Beginning" | "End" | "Anywhere";
    description?: string;
    example_words?: { target: string; cultural_context?: string }[];
  };
  cues: PronunciationCue[];
  onRating: (rating: Rating) => void;
  accessToken: string | undefined;
  onCantSpeak: () => void;
  targetLanguage: Language;
  isNew: boolean;
  showGuide: boolean;
}

export function PronunciationChallenge({
  pattern,
  guide,
  cues,
  onRating,
  accessToken,
  onCantSpeak,
  targetLanguage,
  isNew,
  showGuide,
}: PronunciationChallengeProps) {
  const { bumpBackground } = useBackground();
  const [audioError, setAudioError] = useState(false);

  const leftLabel = isNew ? "Didn't know" : "Forgot";
  const rightLabel = isNew ? "Already knew" : "Remembered";

  const rate = useCallback(
    (rating: Rating) => {
      bumpBackground(30.0);
      window.scrollTo({ top: 0, behavior: "smooth" });
      onRating(rating);
    },
    [bumpBackground, onRating],
  );

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (
        e.target instanceof HTMLInputElement ||
        e.target instanceof HTMLTextAreaElement
      ) {
        return;
      }

      if (e.key === "ArrowRight" && !e.shiftKey) {
        e.preventDefault();
        rate("remembered");
      } else if (e.key === "ArrowLeft") {
        e.preventDefault();
        rate("again");
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [rate]);

  return (
    <div className="flex flex-col flex-1 justify-between">
      <div className="flex flex-col gap-2">
        {/* Guide text above card */}
        {showGuide && (
          <div className="text-center mt-4 text-2xl font-semibold text-muted-foreground animate-fade-in flex flex-row justify-center items-start gap-1">
            <PlayfulArrow direction="down" flipStart size={70} />
            <div>
              Let's practice saying "
              <TargetLanguageText language={targetLanguage}>
                {pattern}
              </TargetLanguageText>
              "
            </div>
            <PlayfulArrow direction="down" size={70} />
          </div>
        )}

        <Card animate className="pt-4 pb-4 pl-4 pr-4">
          <div className="flex flex-col gap-4">
            <div className="flex flex-col items-center gap-1">
              <div className="text-center text-3xl font-bold">
                <TargetLanguageText language={targetLanguage}>
                  {match(guide.position)
                    .with("Beginning", () => `${pattern}___`)
                    .with("End", () => `___${pattern}`)
                    .with("Anywhere", () => pattern)
                    .exhaustive()}
                </TargetLanguageText>
              </div>
              {guide.position !== "Anywhere" && (
                <span className="text-xs text-muted-foreground/80">
                  {match(guide.position)
                    .with(
                      "Beginning",
                      () => "Appears at the beginning of words",
                    )
                    .with("End", () => "Appears at the end of words")
                    .exhaustive()}
                </span>
              )}
            </div>

            {guide.example_words && guide.example_words.length > 0 && (
              <div className="space-y-3">
                <div className="grid gap-3">
                  {guide.example_words.slice(0, 3).map((example, index) => (
                    <PronunciationRow
                      key={index}
                      cue={cues[index]}
                      example={example}
                      pattern={pattern}
                      position={guide.position}
                      targetLanguage={targetLanguage}
                      accessToken={accessToken}
                      onError={() => setAudioError(true)}
                      onSuccess={() => setAudioError(false)}
                    />
                  ))}
                </div>
              </div>
            )}

            {guide.description && (
              <div className="pt-3 border-t border-muted/20">
                <div className="text-sm text-muted-foreground">
                  <Markdown>{guide.description}</Markdown>
                </div>
              </div>
            )}
          </div>
        </Card>
      </div>

      <div className="flex flex-col">
        {/* Guide text above buttons */}
        {showGuide && (
          <div className="text-center mt-4 text-2xl font-semibold text-muted-foreground flex flex-row justify-center items-start animate-fade-in-delayed">
            <PlayfulArrow direction="down" flipStart size={96} />
            <span>How was your pronunciation?</span>
            <PlayfulArrow direction="down" size={96} />
          </div>
        )}

        <div className="mt-4 flex flex-col gap-2 sticky bottom-0">
          {audioError && <AudioErrorBanner onSkip={onCantSpeak} />}
          <CantSpeakButton onClick={onCantSpeak} />
          <div className="grid grid-cols-2">
            <Button
              onClick={() => rate("again")}
              variant="destructive"
              size="lg"
              className="h-14 text-lg rounded-r-none group"
            >
              <span className="relative flex items-center justify-center">
                <kbd className="absolute right-full mr-2 h-6 w-6 text-xs font-semibold border rounded bg-background/20 border-background/40 flex items-center justify-center hide-kbd-mobile opacity-0 group-hover:opacity-100 transition-opacity">
                  <ArrowLeft className="h-3 w-3" />
                </kbd>
                {leftLabel}
              </span>
            </Button>
            <Button
              onClick={() => rate("remembered")}
              variant="default"
              size="lg"
              className="h-14 text-lg rounded-l-none group"
            >
              <span className="relative flex items-center justify-center">
                {rightLabel}
                <kbd className="absolute left-full ml-2 h-6 w-6 text-xs font-semibold border rounded bg-background/20 border-background/40 flex items-center justify-center hide-kbd-mobile opacity-0 group-hover:opacity-100 transition-opacity">
                  <ArrowRight className="h-3 w-3" />
                </kbd>
              </span>
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}

function PronunciationRow({
  cue,
  example,
  pattern,
  position,
  targetLanguage,
  accessToken,
  onError,
  onSuccess,
}: {
  cue: PronunciationCue;
  example: { target: string; cultural_context?: string };
  pattern: string;
  position: "Beginning" | "End" | "Anywhere";
  targetLanguage: Language;
  accessToken: string | undefined;
  onError: () => void;
  onSuccess: () => void;
}) {
  const [positionMs, setPositionMs] = useState<number | null>(null);
  const firstExample = cue.segments.findIndex(
    (segment) => segment.role === "Example",
  );
  let lastExample = -1;
  let current = -1;
  cue.segments.forEach((segment, index) => {
    if (segment.role === "Example") lastExample = index;
    if (
      positionMs !== null &&
      segment.start_ms != null &&
      positionMs >= segment.start_ms
    )
      current = index;
  });

  return (
    <div className="bg-muted/30 rounded p-3 flex items-center justify-between gap-3">
      <div className="flex-1 flex flex-col gap-1">
        <div className="text-base">
          <TargetLanguageText language={targetLanguage}>
            {cue.segments.map((segment, index) => {
              let patternIndex = -1;
              if (segment.role === "Example") {
                const word = segment.text.toLowerCase();
                const needle = pattern.toLowerCase();
                if (
                  position === "Beginning" &&
                  index === firstExample &&
                  word.startsWith(needle)
                ) {
                  patternIndex = 0;
                } else if (
                  position === "End" &&
                  index === lastExample &&
                  word.endsWith(needle)
                ) {
                  patternIndex = segment.text.length - pattern.length;
                } else if (position === "Anywhere") {
                  patternIndex = word.indexOf(needle);
                }
              }
              const timed = positionMs !== null && segment.start_ms != null;
              const unspoken =
                positionMs !== null &&
                segment.start_ms != null &&
                positionMs < segment.start_ms;
              const style =
                segment.role === "Pattern"
                  ? "font-medium"
                  : segment.role === "Connector"
                    ? ""
                    : "font-semibold";
              const color =
                timed && index === current
                  ? "text-primary"
                  : segment.role === "Connector"
                    ? "text-muted-foreground"
                    : "";
              const separated =
                index > 0 &&
                !(
                  segment.role === "Pattern" &&
                  cue.segments[index - 1].role === "Pattern"
                );
              return (
                <span key={index}>
                  {separated && " "}
                  <span
                    className={`${style} transition-[color,opacity] duration-100 ${unspoken ? "opacity-50" : ""} ${color}`}
                  >
                    {patternIndex < 0 ? (
                      segment.text
                    ) : (
                      <>
                        {segment.text.slice(0, patternIndex)}
                        <span className="bg-yellow-500/30 rounded px-0.5">
                          {segment.text.slice(
                            patternIndex,
                            patternIndex + pattern.length,
                          )}
                        </span>
                        {segment.text.slice(patternIndex + pattern.length)}
                      </>
                    )}
                  </span>
                </span>
              );
            })}
          </TargetLanguageText>
        </div>
        {example.cultural_context && (
          <div className="text-xs text-muted-foreground">
            {example.cultural_context}
          </div>
        )}
      </div>
      <AudioButton
        audioRequest={cue.audio}
        accessToken={accessToken}
        autoPlay={false}
        onTimeUpdate={setPositionMs}
        onError={onError}
        onSuccess={onSuccess}
      />
    </div>
  );
}
