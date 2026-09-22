import type {
  CueSegment,
  PronunciationCue,
  PronunciationView,
  Language,
  Rating,
} from "../../../../yap-frontend-rs/pkg";
import Markdown from "react-markdown";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Fragment, useCallback, useEffect, useState } from "react";
import { languageToLangAttr } from "@/lib/utils";
import { AudioButton } from "../AudioButton";
import { CantSpeakButton } from "../CantSpeakButton";
import { AudioErrorBanner } from "../AudioErrorBanner";
import { useBackground } from "../background-context";
import { PlayfulArrow } from "../PlayfulArrow";
import { ArrowLeft, ArrowRight } from "lucide-react";
import { TargetLanguageText } from "../TargetLanguageText";

interface PronunciationChallengeProps {
  view: PronunciationView;
  onRating: (rating: Rating) => void;
  accessToken: string | undefined;
  onCantSpeak: () => void;
  targetLanguage: Language;
  nativeLanguage: Language;
}

export function PronunciationChallenge({
  view,
  onRating,
  accessToken,
  onCantSpeak,
  targetLanguage,
  nativeLanguage,
}: PronunciationChallengeProps) {
  const { bumpBackground } = useBackground();
  const [audioError, setAudioError] = useState(false);

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
        {view.tutorial_prompt && (
          <div className="text-center mt-4 text-2xl font-semibold text-muted-foreground animate-fade-in flex flex-row justify-center items-start gap-1">
            <PlayfulArrow direction="down" flipStart size={70} />
            <div>
              {view.tutorial_prompt.before}
              {view.tutorial_prompt.target && (
                <TargetLanguageText language={targetLanguage}>
                  {view.tutorial_prompt.target}
                </TargetLanguageText>
              )}
              {view.tutorial_prompt.after}
            </div>
            <PlayfulArrow direction="down" size={70} />
          </div>
        )}

        <Card animate className="pt-4 pb-4 pl-4 pr-4">
          <div className="flex flex-col gap-4">
            <div className="flex flex-col items-center gap-1">
              <div className="text-center text-3xl font-bold">
                <TargetLanguageText language={targetLanguage}>
                  {view.positioned_pattern}
                </TargetLanguageText>
              </div>
              {view.position_note && (
                <span className="text-xs text-muted-foreground/80">
                  {view.position_note}
                </span>
              )}
            </div>

            {view.examples.length > 0 && (
              <div className="space-y-3">
                <div className="grid gap-3">
                  {view.examples.map((example, index) => (
                    <PronunciationRow
                      key={index}
                      cue={example.cue}
                      culturalContext={example.cultural_context}
                      pattern={view.pattern}
                      position={view.position}
                      targetLanguage={targetLanguage}
                      nativeLanguage={nativeLanguage}
                      accessToken={accessToken}
                      onError={() => setAudioError(true)}
                      onSuccess={() => setAudioError(false)}
                    />
                  ))}
                </div>
              </div>
            )}

            {view.description && (
              <div className="pt-3 border-t border-muted/20">
                <div className="text-sm text-muted-foreground">
                  <Markdown>{view.description}</Markdown>
                </div>
              </div>
            )}
          </div>
        </Card>
      </div>

      <div className="flex flex-col">
        {/* Guide text above buttons */}
        {view.tutorial_grade_prompt && (
          <div className="text-center mt-4 text-2xl font-semibold text-muted-foreground flex flex-row justify-center items-start animate-fade-in-delayed">
            <PlayfulArrow direction="down" flipStart size={96} />
            <span>{view.tutorial_grade_prompt}</span>
            <PlayfulArrow direction="down" size={96} />
          </div>
        )}

        <div className="mt-4 flex flex-col gap-2 sticky bottom-0">
          {audioError && <AudioErrorBanner onSkip={onCantSpeak} />}
          <CantSpeakButton onClick={onCantSpeak} label={view.cant_speak_label} />
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
                {view.again_label}
              </span>
            </Button>
            <Button
              onClick={() => rate("remembered")}
              variant="default"
              size="lg"
              className="h-14 text-lg rounded-l-none group"
            >
              <span className="relative flex items-center justify-center">
                {view.remembered_label}
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
  culturalContext,
  pattern,
  position,
  targetLanguage,
  nativeLanguage,
  accessToken,
  onError,
  onSuccess,
}: {
  cue: PronunciationCue;
  culturalContext: string | undefined;
  pattern: string;
  position: "Beginning" | "End" | "Anywhere";
  targetLanguage: Language;
  nativeLanguage: Language;
  accessToken: string | undefined;
  onError: () => void;
  onSuccess: () => void;
}) {
  const [positionMs, setPositionMs] = useState<number | null>(null);
  // This row's connector reads as the learner's own "as in" until they play
  // this row's clip, after which it stays as the word the voice used. It's per
  // row because the swap is meant to be explained by the audio just heard.
  const [connectorHeard, setConnectorHeard] = useState(false);
  const firstExample = cue.segments.findIndex(
    (segment) => segment.role === "Example",
  );
  // The connector words are one contiguous run, so the first index is enough
  // to find them and to know where the swap goes.
  const firstConnector = cue.segments.findIndex(
    (segment) => segment.role === "Connector",
  );
  const connectorSegments = cue.segments.filter(
    (segment) => segment.role === "Connector",
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

  // One word as the voice says it: dimmed until the playhead reaches it, lit
  // while it's being said, with the pattern picked out inside example words.
  const spokenWord = (segment: CueSegment, index: number) => {
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
    return (
      <TargetLanguageText language={targetLanguage}>
        <span
          className={`${style} transition-[color,opacity] duration-100 ${unspoken ? "opacity-50" : ""} ${color}`}
        >
          {patternIndex < 0 ? (
            segment.text
          ) : (
            <>
              {segment.text.slice(0, patternIndex)}
              <span className="bg-caution/30 rounded px-0.5">
                {segment.text.slice(
                  patternIndex,
                  patternIndex + pattern.length,
                )}
              </span>
              {segment.text.slice(patternIndex + pattern.length)}
            </>
          )}
        </span>
      </TargetLanguageText>
    );
  };

  // "as in" reads for a learner who has never met the target-language
  // connector; the real word reads once they've heard it. Both sit in the same
  // grid cell, so the cell is as wide as the wider of the two and swapping
  // them moves nothing else on the line.
  const connector = (
    <span className="inline-grid items-baseline justify-items-center px-1 align-baseline">
      <span
        lang={languageToLangAttr(nativeLanguage)}
        aria-hidden={connectorHeard}
        className={`col-start-1 row-start-1 text-muted-foreground transition-opacity duration-300 ${connectorHeard ? "opacity-0" : ""}`}
      >
        {cue.native_connector}
      </span>
      <span
        aria-hidden={!connectorHeard}
        className={`col-start-1 row-start-1 transition-opacity duration-300 ${connectorHeard ? "" : "opacity-0"}`}
      >
        {connectorSegments.map((segment, offset) => (
          <Fragment key={offset}>
            {offset > 0 && " "}
            {spokenWord(segment, firstConnector + offset)}
          </Fragment>
        ))}
      </span>
    </span>
  );

  return (
    <div className="bg-muted/30 rounded p-3 flex items-center justify-between gap-3">
      <div className="flex-1 flex flex-col gap-1">
        <div className="text-base">
          {cue.segments.map((segment, index) => {
            // The connector run renders once, at its first word.
            if (segment.role === "Connector" && index !== firstConnector) {
              return null;
            }
            const separated =
              index > 0 &&
              !(
                segment.role === "Pattern" &&
                cue.segments[index - 1].role === "Pattern"
              );
            return (
              <Fragment key={index}>
                {separated && " "}
                {index === firstConnector
                  ? connector
                  : spokenWord(segment, index)}
              </Fragment>
            );
          })}
        </div>
        {culturalContext && (
          <div className="text-xs text-muted-foreground">
            {culturalContext}
          </div>
        )}
      </div>
      <AudioButton
        audioRequest={cue.audio}
        accessToken={accessToken}
        autoPlay={false}
        onTimeUpdate={(ms) => {
          setPositionMs(ms);
          // The first frame of playback is when the voice actually starts, so
          // the connector turns over as the learner hears it.
          if (ms !== null) setConnectorHeard(true);
        }}
        onError={onError}
        onSuccess={onSuccess}
      />
    </div>
  );
}
