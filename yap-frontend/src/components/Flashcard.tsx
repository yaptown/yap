import {
  type AudioRequest,
  type CardContent,
  type DefinitionView,
  type FlashcardView,
  type Language,
  type Literal,
  type Rating,
} from "../../../yap-frontend-rs/pkg";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { MoreVertical, ArrowLeft, ArrowRight, ArrowDown } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import {
  motion,
  useMotionValue,
  useTransform,
  useAnimation as animationControls,
  type PanInfo,
} from "framer-motion";
import { useCallback, useEffect, useState, type ReactNode } from "react";
import "./Flashcard.css";
import { AudioButton } from "./AudioButton";
import { CantListenButton } from "./CantListenButton";
import { AudioErrorBanner } from "./AudioErrorBanner";
import { toast } from "sonner";
import { match } from "ts-pattern";
import { useBackground } from "./background-context";
import { PlayfulArrow } from "./PlayfulArrow";
import { cn } from "@/lib/utils";
import { highlightTermInSentence } from "@/utils/highlightTermInSentence";
import { TargetLanguageText } from "./TargetLanguageText";
import { MorphemeBreakdown, type BreakdownRow } from "./MorphemeBreakdown";

function gramDisplayText(gram: Literal<string>[]): string {
  return gram
    .map((l) => l.word.text + l.whitespace)
    .join("")
    .trim();
}

interface FlashcardProps {
  audioRequest: AudioRequest | undefined;
  content: CardContent;
  view: FlashcardView;
  onRating?: (rating: Rating) => void;
  accessToken: string | undefined;
  onCantListen?: () => void;
  targetLanguage: Language;
  autoplayed: boolean;
  setAutoplayed: () => void;
  /** Extra dropdown-menu items (e.g. "Report an Issue"), owned by the caller. */
  menuExtras?: ReactNode;
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
      const definition = content.definition;
      const text = gramDisplayText(content.gram);

      if (!definition.is_phrase) {
        const wordPrefix = content.prefix;
        return (
          <h2 className="text-3xl font-semibold">
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
      } else {
        return (
          <h2 className="text-3xl font-semibold">
            <TargetLanguageText language={targetLanguage}>
              {text}
            </TargetLanguageText>
          </h2>
        );
      }
    })
    .exhaustive();
};

const CardBack = ({
  content,
  view,
  targetLanguage,
  accessToken,
}: {
  content: CardContent;
  view: FlashcardView;
  targetLanguage: Language;
  accessToken: string | undefined;
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
                      ? "bg-green-500/10 border border-green-500/20"
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
                      <span className="text-sm text-green-600 ml-2">
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
      const definition = content.definition;
      const term = gramDisplayText(content.gram);

      return (
        <>
          {definition.senses.map((sense, index) => (
            <div
              key={index}
              className={cn(
                "text-left rounded-lg p-4 space-y-2",
                definition.is_phrase ? "bg-muted/30" : "border border-card/50 bg-card/30",
              )}
            >
              <div
                className={cn(
                  "flex items-baseline gap-2",
                  !definition.is_phrase && "justify-between",
                )}
              >
                <span className="text-xl font-medium">{sense.meaning}</span>
                {definition.morphology_label && (
                  <span className="text-sm text-muted-foreground italic">
                    {definition.morphology_label}
                  </span>
                )}
              </div>

              {sense.example && (
                <div
                  className={cn("text-sm", !definition.is_phrase && "space-y-1")}
                >
                  <div className="flex items-start gap-2">
                    <div
                      className={definition.is_phrase ? "order-last" : undefined}
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
                    <div>
                      <p
                        className={cn(
                          "text-muted-foreground italic",
                          !definition.is_phrase && "flex-1",
                        )}
                      >
                        <TargetLanguageText language={targetLanguage}>
                          "{highlightTermInSentence(sense.example.target, term)}"
                        </TargetLanguageText>
                      </p>
                      <p className="text-muted-foreground">
                        "{sense.example.native}"
                      </p>
                    </div>
                  </div>
                </div>
              )}
            </div>
          ))}
        </>
      );
    })
    .exhaustive();
};

export const Flashcard = function Flashcard({
  audioRequest,
  content,
  view,
  onRating,
  accessToken,
  onCantListen,
  targetLanguage,
  autoplayed,
  setAutoplayed,
  menuExtras,
}: FlashcardProps) {
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
      if (onRating) {
        bumpBackground(30.0);
        window.scrollTo({ top: 0, behavior: "smooth" });
        onRating("remembered");
      }
    } else if (info.offset.x < -threshold && info.velocity.x < 0) {
      // Swiped left - Again
      await controls.start({
        x: -300,
        transition: { duration: 0.2 },
      });
      if (onRating) {
        bumpBackground(30.0);
        window.scrollTo({ top: 0, behavior: "smooth" });
        onRating("again");
      }
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
        if (onRating) {
          bumpBackground(30.0);
          window.scrollTo({ top: 0, behavior: "smooth" });
          onRating("remembered");
        }
      }
      // Mark as "again": ←
      else if (canGrade && e.key === "ArrowLeft") {
        e.preventDefault();
        if (onRating) {
          bumpBackground(30.0);
          window.scrollTo({ top: 0, behavior: "smooth" });
          onRating("again");
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [showAnswer, canGrade, toggleAnswer, onRating, bumpBackground]);

  const copyWord = () => {
    const word = match(content)
      .with({ type: "Gram" }, (c) => gramDisplayText(c.gram))
      .with({ type: "Listening" }, (c) =>
        c.possible_grams.length > 0
          ? gramDisplayText(c.possible_grams[0][1])
          : undefined,
      )
      .exhaustive();

    if (word) {
      navigator.clipboard
        .writeText(word)
        .then(() => toast("Copied to clipboard"))
        .catch(() => toast("Failed to copy"));
    } else {
      toast("No word to copy");
    }
  };

  return (
    <div className="flex flex-col flex-1 justify-between">
      <div className="flex flex-col gap-2">
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
            className={`pt-3 pb-3 pl-3 pr-3 cursor-pointer transition-all hover:shadow-lg overflow-hidden flashcard h-full gap-0 ${
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
              className="absolute inset-0 bg-red-500/20 pointer-events-none"
              style={{ opacity: leftOverlayOpacity }}
            />
            <motion.div
              className="absolute inset-0 bg-green-500/20 pointer-events-none"
              style={{ opacity: rightOverlayOpacity }}
            />

            {/* Swipe indicators */}
            <motion.div
              className="absolute top-8 left-8 text-red-500 font-bold text-2xl rotate-[-30deg] pointer-events-none"
              style={{ opacity: leftOverlayOpacity }}
            >
              {view.again_label.toUpperCase()}
            </motion.div>
            <motion.div
              className="absolute top-8 right-8 text-green-500 font-bold text-2xl rotate-[30deg] pointer-events-none"
              style={{ opacity: rightOverlayOpacity }}
            >
              {view.remembered_label.toUpperCase()}
            </motion.div>

            <div className="text-center relative z-10 flex flex-col gap-6">
              <div className="justify-center gap-2 flex flex-col items-center w-full">
                <div
                  className={`relative flex items-center w-full ${
                    content.type === "Listening"
                      ? "justify-center"
                      : "justify-between"
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
                      {audioRequest ? (
                        <AudioButton
                          audioRequest={audioRequest}
                          accessToken={accessToken}
                          autoPlay={showAnswer}
                          autoplayed={autoplayed}
                          setAutoplayed={setAutoplayed}
                          onError={() => setAudioError(true)}
                          onSuccess={() => setAudioError(false)}
                        />
                      ) : (
                        <div className="w-10" /> /* Spacer to keep content centered */
                      )}

                      <CardFront
                        content={content}
                        targetLanguage={targetLanguage}
                      />
                    </>
                  )}

                  <div
                    className={
                      content.type === "Listening"
                        ? "absolute right-0 top-0"
                        : ""
                    }
                  >
                    {onRating ? (
                      <DropdownMenu>
                        <DropdownMenuTrigger asChild>
                          <Button
                            variant="ghost"
                            size="icon"
                            className="h-10 w-10"
                          >
                            <MoreVertical className="h-6 w-6 size--xl" />
                          </Button>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent align="end">
                          {view.menu_grades.map((grade) => (
                            <DropdownMenuItem
                              key={grade.rating}
                              disabled={!canGrade}
                              onClick={() => {
                                if (!canGrade) return;
                                bumpBackground(30.0);
                                onRating(grade.rating);
                              }}
                            >
                              {grade.label}
                            </DropdownMenuItem>
                          ))}
                          <DropdownMenuItem onClick={copyWord}>
                            Copy word
                          </DropdownMenuItem>
                          {menuExtras}
                        </DropdownMenuContent>
                      </DropdownMenu>
                    ) : (
                      <div className="w-8" />
                    )}
                  </div>
                </div>
                {view.subtitle && (
                  <span className="text-sm text-muted-foreground">{view.subtitle}</span>
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
                  />
                </div>
              ) : (
                <div className="flex flex-col items-center gap-2">
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
              <div className="grid grid-cols-2">
                <Button
                  onClick={() => {
                    if (!canGrade) return;
                    bumpBackground(30.0);
                    window.scrollTo({ top: 0, behavior: "smooth" });
                    onRating("again");
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
                    {view.again_label}
                  </span>
                </Button>
                <Button
                  onClick={() => {
                    if (!canGrade) return;
                    bumpBackground(30.0);
                    window.scrollTo({ top: 0, behavior: "smooth" });
                    onRating("remembered");
                  }}
                  variant="default"
                  size="lg"
                  className="h-14 text-lg rounded-l-none group"
                  disabled={!canGrade}
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
        )}
      </div>
    </div>
  );
};
