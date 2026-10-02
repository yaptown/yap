import {
  type AudioRequest,
  type CardContent,
  type DefinitionSense,
  type DefinitionView,
  type FlashcardView,
  type Language,
  type Literal,
  type Rating,
} from "../../../../yap-frontend-rs/pkg";
import { ArrowLeft, ArrowRight, ArrowDown, Check, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import {
  motion,
  useMotionValue,
  useTransform,
  useAnimation as animationControls,
  type PanInfo,
} from "framer-motion";
import { useCallback, useEffect, useState } from "react";
import "./FlashcardChallenge.css";
import { ReportIssueLink } from "./translation-verdict";
import { AudioButton } from "../../audio/AudioButton";
import { CantListenButton } from "./CantListenButton";
import { AudioErrorBanner } from "../../audio/AudioErrorBanner";
import { toast } from "sonner";
import { match } from "ts-pattern";
import { useBackground } from "../../components/background-context";
import { PlayfulArrow } from "./PlayfulArrow";
import { cn } from "@/lib/utils";
import { highlightTermInSentence } from "@/utils/highlightTermInSentence";
import { TargetLanguageText } from "../../components/TargetLanguageText";
import { MorphemeBreakdown, type BreakdownRow } from "../MorphemeBreakdown";

function gramDisplayText(gram: Literal<string>[]): string {
  return gram
    .map((l) => l.word.text + l.whitespace)
    .join("")
    .trim();
}

interface FlashcardChallengeProps {
  audioRequest: AudioRequest | undefined;
  content: CardContent;
  view: FlashcardView;
  /** `meaningRatings` holds one rating per meaning of a written card; see
   * `flashcardReviews`. */
  onRating?: (rating: Rating, meaningRatings?: Rating[]) => void;
  accessToken: string | undefined;
  onCantListen?: () => void;
  targetLanguage: Language;
  autoplayed: boolean;
  setAutoplayed: () => void;
  /** Extra dropdown-menu items (e.g. "Report an Issue"), owned by the caller. */
  reportIssue?: { label: string; onClick: () => void };
}

const CardFront = ({
  content,
  targetLanguage,
}: {
  content: CardContent;
  targetLanguage: Language;
}) => {
  return match(content)
    .with({ type: "Listening" }, () => {
      return null;
    })
    .with({ type: "Gram" }, (content) => {
      const text = gramDisplayText(content.gram);
      const wordPrefix = content.prefix;
      return (
        <h2 className="text-4xl font-bold">
          <TargetLanguageText language={targetLanguage}>
            {wordPrefix && (
              <span className="text-muted-foreground/60">
                {wordPrefix.prefix}
                {wordPrefix.separator}
              </span>
            )}
            {text}
          </TargetLanguageText>
        </h2>
      );
    })
    .exhaustive();
};

const CardBack = ({
  content,
  view,
  targetLanguage,
  accessToken,
  marks,
  onMark,
}: {
  content: CardContent;
  view: FlashcardView;
  targetLanguage: Language;
  accessToken: string | undefined;
  marks: (Rating | undefined)[];
  onMark: (index: number, rating: Rating | undefined) => void;
}) => {
  return match(content)
    .with({ type: "Listening" }, (content) => {
      const possibleGrams = content.possible_grams;

      const definitionsGloss = (definitions: DefinitionView[]): string =>
        definitions
          .flatMap((definition) => definition.senses.map((sense) => sense.meaning))
          .filter(Boolean)
          .join(", ");

      if (possibleGrams.length === 1) {
        const [, gram, definitions] = possibleGrams[0];
        const gloss = definitionsGloss(definitions);
        return (
          <div className="space-y-2">
            <div className="text-3xl font-medium">
              <TargetLanguageText language={targetLanguage}>
                {gramDisplayText(gram)}
              </TargetLanguageText>
            </div>
            {gloss && (
              <div className="text-lg text-muted-foreground">{gloss}</div>
            )}
          </div>
        );
      }

      return (
        <div className="space-y-4">
          {view.listening_header && (
            <div className="text-sm text-muted-foreground">
              {view.listening_header}
            </div>
          )}
          <div className="grid grid-cols-2 gap-2">
            {possibleGrams.map(([isKnown, gram, definitions], index: number) => {
              const gloss = definitionsGloss(definitions);
              return (
                <div
                  key={index}
                  className={`text-left p-2 rounded-md ${
                    isKnown
                      ? "bg-positive-surface border border-positive-border"
                      : "bg-muted/30 border border-muted/20"
                  }`}
                >
                  <div>
                    <span className="text-lg">
                      <TargetLanguageText language={targetLanguage}>
                        {gramDisplayText(gram)}
                      </TargetLanguageText>
                    </span>
                    {isKnown && (
                      <span className="text-sm text-positive-foreground ml-2">
                        {view.known_label}
                      </span>
                    )}
                  </div>
                  {gloss && (
                    <div className="text-sm text-muted-foreground">
                      {gloss}
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        </div>
      );
    })
    .with({ type: "Gram" }, (content) => {
      const term = gramDisplayText(content.gram);

      if (content.meanings.length === 1) {
        const [meaning] = content.meanings;
        return (
          <div className="flex flex-col gap-2">
            {meaning.label && <MeaningLabel>{meaning.label}</MeaningLabel>}
            {meaning.definition.senses.map((sense, index) => (
              <div
                key={index}
                className={cn(
                  "text-left rounded-lg p-4",
                  meaning.definition.is_phrase
                    ? "bg-muted/30"
                    : "border border-card/50 bg-card/30",
                )}
              >
                <Sense
                  sense={sense}
                  definition={meaning.definition}
                  term={term}
                  targetLanguage={targetLanguage}
                  accessToken={accessToken}
                  showMorphology
                />
              </div>
            ))}
          </div>
        );
      }

      return (
        <div className="flex flex-col gap-3">
          {content.meanings.map((meaning, index) => {
            const mark = marks[index];
            return (
              <div
                key={index}
                className={cn(
                  "text-left rounded-lg border p-4 flex flex-col gap-3 transition-colors duration-200",
                  mark === "again"
                    ? "border-negative/50 bg-negative/15"
                    : mark === "remembered"
                      ? "border-positive/50 bg-positive/15"
                      : "border-card/50 bg-card/30",
                )}
              >
                <div className="flex items-center justify-between gap-3">
                  <div className="flex flex-wrap items-center gap-x-2 gap-y-1 min-w-0">
                    {meaning.label && <MeaningLabel>{meaning.label}</MeaningLabel>}
                    {meaning.is_new && view.new_label && (
                      <span className="rounded-full bg-primary/15 px-2 py-0.5 text-xs font-medium text-primary">
                        {view.new_label}
                      </span>
                    )}
                  </div>
                  <MarkToggle
                    mark={mark}
                    onMark={(rating) => onMark(index, rating)}
                  />
                </div>
                {meaning.definition.senses.map((sense, senseIndex) => (
                  <Sense
                    key={senseIndex}
                    sense={sense}
                    definition={meaning.definition}
                    term={meaning.definition.headword}
                    targetLanguage={targetLanguage}
                    accessToken={accessToken}
                    showMorphology
                    stacked
                  />
                ))}
              </div>
            );
          })}
        </div>
      );
    })
    .exhaustive();
};

const MeaningLabel = ({ children }: { children: React.ReactNode }) => (
  <span className="self-start text-left text-xs font-medium uppercase tracking-wider text-muted-foreground">
    {children}
  </span>
);

/// One meaning's gloss and example sentence. A lone phrase keeps its audio
/// after the example; `stacked` rows all lead with it so they line up.
const Sense = ({
  sense,
  definition,
  term,
  targetLanguage,
  accessToken,
  showMorphology = false,
  stacked = false,
}: {
  sense: DefinitionSense;
  definition: DefinitionView;
  term: string;
  targetLanguage: Language;
  accessToken: string | undefined;
  showMorphology?: boolean;
  stacked?: boolean;
}) => (
  <div className="flex flex-col gap-2">
    <div
      className={cn(
        "flex items-baseline gap-2",
        (stacked || !definition.is_phrase) && "justify-between",
      )}
    >
      <span className="text-xl font-medium">{sense.meaning}</span>
      {showMorphology && definition.morphology_label && (
        <span className="text-sm text-muted-foreground italic text-right">
          {definition.morphology_label}
        </span>
      )}
    </div>

    {sense.example && (
      <div className="flex items-start gap-2 text-sm">
        <div
          className={definition.is_phrase && !stacked ? "order-last" : undefined}
          onClick={(e) => e.stopPropagation()}
        >
          <AudioButton
            audioRequest={{
              request: {
                text: sense.example.target,
                language: targetLanguage,
              },
              provider: "ElevenLabs",
            }}
            accessToken={accessToken}
            className="h-8 w-8"
            size="icon"
          />
        </div>
        <div className="flex flex-col gap-1">
          <p className="text-muted-foreground italic">
            <TargetLanguageText language={targetLanguage}>
              "{highlightTermInSentence(sense.example.target, term)}"
            </TargetLanguageText>
          </p>
          <p className="text-muted-foreground">"{sense.example.native}"</p>
        </div>
      </div>
    )}
  </div>
);

/// Per-meaning ✗ / ✓. Tapping the active mark clears it again.
const MarkToggle = ({
  mark,
  onMark,
}: {
  mark: Rating | undefined;
  onMark: (rating: Rating | undefined) => void;
}) => {
  const option = (rating: Rating, Icon: typeof X, active: string) => (
    <button
      type="button"
      aria-pressed={mark === rating}
      aria-label={rating === "again" ? "Forgot" : "Remembered"}
      onClick={(e) => {
        e.stopPropagation();
        onMark(mark === rating ? undefined : rating);
      }}
      onPointerDownCapture={(e) => e.stopPropagation()}
      className={cn(
        "h-9 w-9 rounded-full border flex items-center justify-center transition-colors",
        mark === rating
          ? active
          : "border-border bg-background/60 text-muted-foreground hover:text-foreground",
      )}
    >
      <Icon className="h-4 w-4" strokeWidth={2.5} />
    </button>
  );
  return (
    <div className="flex shrink-0 gap-2">
      {option("again", X, "border-transparent bg-negative text-white")}
      {option("remembered", Check, "border-transparent bg-positive text-white")}
    </div>
  );
};

export const FlashcardChallenge = function FlashcardChallenge({
  audioRequest,
  content,
  view,
  onRating,
  accessToken,
  onCantListen,
  targetLanguage,
  autoplayed,
  setAutoplayed,
  reportIssue,
}: FlashcardChallengeProps) {
  const x = useMotionValue(0);
  const controls = animationControls();
  const [isDragging, setIsDragging] = useState(false);
  const [showAnswer, setShowAnswer] = useState(false);
  const [hasBeenOpened, setHasBeenOpened] = useState(false);
  const [audioError, setAudioError] = useState(false);
  const { bumpBackground } = useBackground();

  const toggleAnswer = useCallback(() => {
    setShowAnswer((shown) => !shown);
    setHasBeenOpened(true);
  }, []);

  const canGrade = hasBeenOpened || showAnswer || !view.require_answer_reveal;

  // A written card with several meanings can be graded meaning by meaning;
  // the grade buttons then apply to whichever meanings are left unmarked.
  const meaningCount = content.type === "Gram" ? content.meanings.length : 0;
  const [marks, setMarks] = useState<(Rating | undefined)[]>(() =>
    Array.from({ length: meaningCount }, () => undefined),
  );
  const markedCount = marks.filter(Boolean).length;
  const allMarked = meaningCount > 1 && markedCount === meaningCount;
  const againLabel =
    (allMarked
      ? view.continue_label
      : markedCount > 0
        ? view.again_rest_label
        : undefined) ?? view.again_label;
  const rememberedLabel =
    (allMarked
      ? view.continue_label
      : markedCount > 0
        ? view.remembered_rest_label
        : undefined) ?? view.remembered_label;
  const onMark = useCallback((index: number, rating: Rating | undefined) => {
    setMarks((marks) => marks.map((mark, i) => (i === index ? rating : mark)));
  }, []);

  const grade = useCallback(
    (rest: Rating) => {
      if (!onRating) return;
      bumpBackground(30.0);
      window.scrollTo({ top: 0, behavior: "smooth" });
      if (content.type === "Gram") {
        const meaningRatings = content.meanings.map((_, i) => marks[i] ?? rest);
        const overall = meaningRatings.every((rating) => rating === "again")
          ? "again"
          : "remembered";
        onRating(overall, meaningRatings);
      } else {
        onRating(rest);
      }
    },
    [onRating, bumpBackground, content, marks],
  );

  const rotate = useTransform(x, [-200, 200], [-30, 30]);

  // Color overlay for visual feedback
  const leftOverlayOpacity = useTransform(x, [-200, 0], [1, 0]);
  const rightOverlayOpacity = useTransform(x, [0, 200], [0, 1]);

  const handleDragEnd = async (
    _event: MouseEvent | TouchEvent | PointerEvent,
    info: PanInfo,
  ) => {
    setIsDragging(false);
    const threshold = 100;

    if (!canGrade) {
      controls.start({
        x: 0,
        transition: { type: "spring", stiffness: 300, damping: 20 },
      });
      return;
    }

    if (info.offset.x > threshold && info.velocity.x > 0) {
      // Swiped right - "remembered"
      await controls.start({
        x: 300,
        transition: { duration: 0.2 },
      });
      grade("remembered");
    } else if (info.offset.x < -threshold && info.velocity.x < 0) {
      // Swiped left - Again
      await controls.start({
        x: -300,
        transition: { duration: 0.2 },
      });
      grade("again");
    } else {
      // Not enough swipe - snap back
      controls.start({
        x: 0,
        transition: { type: "spring", stiffness: 300, damping: 20 },
      });
    }
  };

  // Reset position and animate in
  useEffect(() => {
    // Reset to initial state instantly, then animate in
    controls.set({ x: 0, scale: 0.95 });
    controls.start({
      x: 0,
      scale: 1,
      transition: {
        duration: 0.3,
        ease: "easeOut",
      },
    });
  }, [controls]);

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ignore if user is typing in an input
      if (
        e.target instanceof HTMLInputElement ||
        e.target instanceof HTMLTextAreaElement
      ) {
        return;
      }

      if (e.key === "Enter") {
        e.preventDefault();
        toast("Use the arrow keys");
        return;
      }

      if (["1", "2", "3", "4"].includes(e.key)) {
        e.preventDefault();
        toast("Use the arrow keys");
        return;
      }

      if (e.key === "ArrowDown" || e.key === "ArrowUp") {
        e.preventDefault();
      }

      // Show answer: Space / ↓ / j (when answer is hidden)
      if (
        !showAnswer &&
        (e.key === " " || e.key === "ArrowDown" || e.key === "j")
      ) {
        e.preventDefault();
        toggleAnswer();
      }
      // Hide answer: ↑ / k
      else if (showAnswer && (e.key === "ArrowUp" || e.key === "k")) {
        e.preventDefault();
        toggleAnswer();
      }
      // Mark as remembered: →
      else if (canGrade && e.key === "ArrowRight" && !e.shiftKey) {
        e.preventDefault();
        grade("remembered");
      }
      // Mark as "again": ← (Continue has only the → key)
      else if (canGrade && e.key === "ArrowLeft" && !allMarked) {
        e.preventDefault();
        grade("again");
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [showAnswer, canGrade, toggleAnswer, grade, allMarked]);

  return (
    <div className="flex flex-col flex-1 justify-between">
      <div className="flex flex-1 flex-col gap-2">
        {/* Tutorial text above card */}
        {view.tutorial_prompt && (
          <div
            className={cn(
              "grid transition-all duration-300",
              showAnswer
                ? "grid-rows-[0fr] opacity-0"
                : "grid-rows-[1fr] opacity-100",
            )}
          >
            <div className="overflow-hidden">
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
            </div>
          </div>
        )}

        <motion.div
          className="relative w-full"
          drag="x"
          dragConstraints={{ left: 0, right: 0 }}
          onDragStart={() => setIsDragging(true)}
          onDragEnd={handleDragEnd}
          animate={controls}
          style={{ x, rotate }}
        >
          <Card
            className={`p-5 cursor-pointer transition-all hover:shadow-lg overflow-hidden flashcard h-full gap-0 ${
              !showAnswer ? "spin-on-hover" : ""
            }`}
            onClick={() => {
              if (!isDragging) {
                toggleAnswer();
              }
            }}
            animate
          >
            {/* Swipe feedback overlays */}
            <motion.div
              className="absolute inset-0 bg-negative/20 pointer-events-none"
              style={{ opacity: leftOverlayOpacity }}
            />
            <motion.div
              className="absolute inset-0 bg-positive/20 pointer-events-none"
              style={{ opacity: rightOverlayOpacity }}
            />

            {/* Swipe indicators */}
            <motion.div
              className="absolute top-8 left-8 text-negative font-bold text-2xl rotate-[-30deg] pointer-events-none"
              style={{ opacity: leftOverlayOpacity }}
            >
              {againLabel.toUpperCase()}
            </motion.div>
            <motion.div
              className="absolute top-8 right-8 text-positive font-bold text-2xl rotate-[30deg] pointer-events-none"
              style={{ opacity: rightOverlayOpacity }}
            >
              {rememberedLabel.toUpperCase()}
            </motion.div>

            <div className="text-center relative z-10 flex flex-col gap-6">
              <div className="justify-center gap-2 flex flex-col items-center w-full">
                <div
                  className={`flex w-full items-center ${
                    content.type === "Listening"
                      ? "justify-center"
                      : "justify-between gap-3"
                  }`}
                  onClick={(e) => e.stopPropagation()}
                >
                  {content.type === "Listening" ? (
                    audioRequest && (
                      <AudioButton
                        audioRequest={audioRequest}
                        accessToken={accessToken}
                        autoPlay
                        autoplayed={autoplayed}
                        setAutoplayed={setAutoplayed}
                        onError={() => setAudioError(true)}
                        onSuccess={() => setAudioError(false)}
                        visualizer
                      />
                    )
                  ) : (
                    <>
                      <div className="text-left">
                        <CardFront
                          content={content}
                          targetLanguage={targetLanguage}
                        />
                      </div>
                      {audioRequest && (
                        <AudioButton
                          audioRequest={audioRequest}
                          accessToken={accessToken}
                          autoPlay={showAnswer}
                          autoplayed={autoplayed}
                          setAutoplayed={setAutoplayed}
                          onError={() => setAudioError(true)}
                          onSuccess={() => setAudioError(false)}
                          variant="default"
                          className="h-14 w-14 shrink-0 rounded-full bg-primary hover:bg-primary/90 shadow-lg shadow-primary/30 [&_svg]:fill-current"
                        />
                      )}
                    </>
                  )}
                </div>
                {view.subtitle && (
                  <span
                    className={`text-sm text-muted-foreground ${content.type === "Listening" ? "" : "self-start"}`}
                  >
                    {view.subtitle}
                  </span>
                )}
              </div>

              <hr className="" />

              {showAnswer ? (
                <div className="space-y-6 animate-feedback-in">
                  <CardBack
                    content={content}
                    view={view}
                    targetLanguage={targetLanguage}
                    accessToken={accessToken}
                    marks={marks}
                    onMark={onMark}
                  />
                </div>
              ) : (
                <div
                  className={`flex gap-2 ${
                    content.type === "Listening"
                      ? "flex-col items-center"
                      : "items-center"
                  }`}
                >
                  <div
                    className={` ${
                      view.require_answer_reveal ? "font-bold" : "text-muted-foreground"
                    }`}
                  >
                    {view.reveal_label}
                  </div>
                  <kbd className="h-6 w-6 text-xs font-semibold border rounded bg-muted/20 border flex items-center justify-center hide-kbd-border-mobile">
                    <ArrowDown className="h-3 w-3 text-muted-foreground" />
                  </kbd>
                </div>
              )}
            </div>
          </Card>
        </motion.div>

        {/* Breakdown (morphemes or words), shown after the card is revealed */}
        {showAnswer &&
          content.type === "Gram" &&
          content.breakdown &&
          content.breakdown.length > 0 && (
            <MorphemeBreakdown
              breakdown={content.breakdown as BreakdownRow[]}
              targetLanguage={targetLanguage}
              baseDelay={1.5}
              className="mt-6 flex flex-col items-center"
            />
          )}

        {/* Tutorial text below card */}
        {view.tutorial_hidden_hint && !showAnswer && (
          <div className="text-center mt-2 text-2xl font-semibold text-muted-foreground flex flex-row justify-center items-end animate-fade-in-delayed">
            <PlayfulArrow direction="up" size={70} />
            <span>{view.tutorial_hidden_hint}</span>
            <PlayfulArrow direction="up" flipStart size={70} />
          </div>
        )}

        {reportIssue && <ReportIssueLink {...reportIssue} />}
      </div>

      <div className="flex flex-col sticky bottom-0">
        {/* Tutorial text above buttons */}
        {view.tutorial_revealed_hint && showAnswer && (
          <div className="text-center mt-4 text-2xl font-semibold text-muted-foreground flex flex-row justify-center items-start animate-fade-in-delayed">
            <PlayfulArrow direction="down" flipStart size={96} />
            <span>{view.tutorial_revealed_hint}</span>
            <PlayfulArrow direction="down" size={96} />
          </div>
        )}

        {onRating && (
          <div
            className={`mt-4 flex flex-col gap-2 transition-opacity duration-300`}
          >
            {!showAnswer && (
              <>
                {audioError && onCantListen && content.type === "Listening" && (
                  <AudioErrorBanner onSkip={onCantListen} />
                )}
                {onCantListen && view.cant_listen_label && (
                  <CantListenButton onClick={onCantListen} label={view.cant_listen_label} />
                )}
              </>
            )}
            <div className={!canGrade ? "hidden" : "quick-fade-in"}>
              <div className={allMarked ? "grid" : "grid grid-cols-2"}>
                  {!allMarked && (
                    <Button
                      onClick={() => {
                        if (!canGrade) return;
                        grade("again");
                      }}
                      variant="destructive"
                      size="lg"
                      className="h-14 text-lg rounded-r-none group"
                      disabled={!canGrade}
                    >
                      <span className="relative flex items-center justify-center">
                        <kbd className="absolute right-full mr-2 h-6 w-6 text-xs font-semibold border rounded bg-background/20 border-background/40 flex items-center justify-center hide-kbd-mobile opacity-0 group-hover:opacity-100 transition-opacity">
                          <ArrowLeft className="h-3 w-3" />
                        </kbd>
                        {againLabel}
                      </span>
                    </Button>
                  )}
                  <Button
                    onClick={() => {
                      if (!canGrade) return;
                      grade("remembered");
                    }}
                    variant="default"
                    size="lg"
                    className={cn(
                      "h-14 text-lg group",
                      !allMarked && "rounded-l-none",
                    )}
                    disabled={!canGrade}
                  >
                    <span className="relative flex items-center justify-center">
                      {rememberedLabel}
                      <kbd className="absolute left-full ml-2 h-6 w-6 text-xs font-semibold border rounded bg-background/20 border-background/40 flex items-center justify-center hide-kbd-mobile opacity-0 group-hover:opacity-100 transition-opacity">
                        <ArrowRight className="h-3 w-3" />
                      </kbd>
                    </span>
                  </Button>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
