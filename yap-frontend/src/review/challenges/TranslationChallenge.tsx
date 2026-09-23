import {
  useState,
  useEffect,
  useRef,
  useCallback,
  forwardRef,
  useImperativeHandle,
  useMemo,
} from "react";
import { PendingReview } from "@/review/challenges/pending-review";
import { getMovieMetadata } from "@/lib/movie-cache";
import { reportAutogradeFailure } from "@/core/instrument";
import { MoviePosterGrid } from "./MoviePosterGrid";
import { ProperNounGroups } from "./ProperNounGroups";
import {
  type TranslateComprehensibleSentence,
  type ManualTranslationGrade,
  type DefinitionView,
  autograde_translation,
  translation_pending_slot,
  translation_start,
  translation_resume,
  translation_transition,
  translation_view,
  type TranslationState,
  type TranslationEvent,
  type TranslationStep,
  type TranslationGradeItemView,
  get_app_version,
  type Language,
  type Deck,
  type Heteronym,
} from "../../../../yap-frontend-rs/pkg/yap_frontend_rs";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import {
  motion,
  useMotionValue,
  useTransform,
  useAnimation as animationControls,
  type PanInfo,
} from "framer-motion";
import { Check, X, MoreVertical } from "lucide-react";
import { AudioButton } from "../../audio/AudioButton";
import { VideoClipPlayer } from "../../audio/VideoClipPlayer";
import { ReportIssueModal } from "./ReportIssueModal";
import { playSoundEffect } from "@/lib/sound-effects";
import { useBackground } from "../../components/background-context";
import { languageToLangAttr } from "@/lib/utils";
import { Textarea } from "../../components/ui/textarea";
import { TargetLanguageText } from "../../components/TargetLanguageText";
import {
  MorphemeBreakdown,
  type BreakdownRow,
} from "../MorphemeBreakdown";
import {
  ChallengeSentence,
  CorrectTranslation,
  FeedbackSkeleton,
  TranslationVerdict,
  YourTranslation,
  type TranslationVerdictData,
} from "./translation-verdict";

interface SentenceChallengeProps {
  sentence: TranslateComprehensibleSentence;
  initialState?: TranslationState;
  onComplete: (
    grade:
      | ManualTranslationGrade
      | { perfect: string | null },
    heteronymsTapped: Heteronym<string>[],
    submission: string,
    completedAtMs: number,
  ) => boolean;
  pendingReviewScope: string;
  accessToken: string | undefined;
  targetLanguage: Language;
  nativeLanguage: Language;
  autoplayed: boolean;
  setAutoplayed: () => void;
  deck: Deck;
  totalReviewsCompleted: bigint;
}

type GradeItem = TranslationGradeItemView;

interface SwipeablePhraseProps {
  item: GradeItem;
  onSwipe: (remembered: boolean) => void;
  isSelected?: boolean;
  targetLanguage: Language;
}

export interface SwipeableWordHandle {
  handleButtonClick: (remembered: boolean) => void;
}

const SwipeablePhrase = forwardRef<SwipeableWordHandle, SwipeablePhraseProps>(
  ({ item, onSwipe, isSelected = false, targetLanguage }, ref) => {
    const status = item.grade === undefined ? undefined : item.grade === "Remembered";
    const x = useMotionValue(0);
    const controls = animationControls();
    const { bumpBackground } = useBackground();

    // Interpolate only the opacity; CSS resolves the shared palette hue.
    const background = useTransform(x, (value) => {
      const color = value < 0 ? "--negative" : "--positive";
      const opacity = Math.min(Math.abs(value) / 150, 1) * 20;
      return `color-mix(in oklch, var(${color}) ${opacity}%, transparent)`;
    });

    const handleDragEnd = async (
      _event: MouseEvent | TouchEvent | PointerEvent,
      info: PanInfo,
    ) => {
      const velocityThreshold = 5;
      const positionThreshold = 50;

      if (
        info.velocity.x < -velocityThreshold ||
        (info.velocity.x > velocityThreshold &&
          info.offset.x < -positionThreshold)
      ) {
        await controls.start({ x: -60 });
        onSwipe(false);
      } else if (
        info.velocity.x > velocityThreshold ||
        (info.velocity.x < -velocityThreshold &&
          info.offset.x > positionThreshold)
      ) {
        await controls.start({ x: 60 });
        onSwipe(true);
      }
    };

    const handleButtonClick = useCallback(
      async (remembered: boolean) => {
        bumpBackground(30.0);
        if (remembered) {
          await controls.start({ x: 60 });
          onSwipe(true);
        } else {
          await controls.start({ x: -60 });
          onSwipe(false);
        }
      },
      [controls, onSwipe, bumpBackground],
    );

    useImperativeHandle(
      ref,
      () => ({
        handleButtonClick,
      }),
      [handleButtonClick],
    );

    useEffect(() => {
      if (status === true) {
        controls.start({ x: 60 });
      } else if (status === false) {
        controls.start({ x: -60 });
      } else {
        controls.start({ x: 0 });
      }
    }, [status, controls]);

    return (
      <motion.div
        className="relative flex items-center gap-3"
        initial={{ opacity: 0, scale: 0.8 }}
        animate={{ opacity: 1, scale: 1 }}
        transition={{ duration: 0.2 }}
        data-word-index
      >
        <button
          onClick={() => handleButtonClick(false)}
          className="p-2 rounded-full hover:bg-negative-surface transition-colors"
          aria-label="Mark as Forgot"
        >
          <X className="w-5 h-5 text-negative" />
        </button>

        <div className="flex-1 relative overflow-hidden">
          <motion.div
            drag="x"
            dragConstraints={{ left: -60, right: 60 }}
            onDragEnd={handleDragEnd}
            style={{ x, background }}
            animate={controls}
            className={`relative px-6 py-3 bg-card border rounded-lg cursor-grab active:cursor-grabbing select-none ${
              isSelected ? "ring-2 ring-primary" : ""
            }`}
          >
            <p className="text-lg font-medium text-center">
              <TargetLanguageText language={targetLanguage}>
                {item.label}
              </TargetLanguageText>
            </p>
          </motion.div>
        </div>

        <button
          onClick={() => handleButtonClick(true)}
          className="p-2 rounded-full hover:bg-positive-surface transition-colors"
          aria-label="Mark as remembered"
        >
          <Check className="w-5 h-5 text-positive" />
        </button>
      </motion.div>
    );
  },
);

SwipeablePhrase.displayName = "SwipeablePhrase";

interface PhraseStatusesProps {
  selectedPhraseIndex: number;
  setSelectedPhraseIndex: (index: number) => void;
  gradeItems: GradeItem[];
  phraseRefs: React.RefObject<Map<number, SwipeableWordHandle>>;
  handleGradeSwipe: (index: number, remembered: boolean) => void;
  openByDefault: boolean;
  title: string;
  subtitle: string;
  targetLanguage: Language;
}

function PhraseStatuses({
  selectedPhraseIndex,
  setSelectedPhraseIndex,
  gradeItems,
  phraseRefs,
  handleGradeSwipe,
  openByDefault,
  title,
  subtitle,
  targetLanguage,
}: PhraseStatusesProps) {
  const [isAnswerOpen, setIsAnswerOpen] = useState(openByDefault);

  useEffect(() => {
    if (isAnswerOpen && selectedPhraseIndex === -1 && gradeItems.length > 0) {
      setSelectedPhraseIndex(0);
    }
  }, [
    isAnswerOpen,
    selectedPhraseIndex,
    gradeItems.length,
    setSelectedPhraseIndex,
  ]);

  return (
    <Collapsible open={isAnswerOpen} onOpenChange={setIsAnswerOpen}>
      <CollapsibleTrigger asChild>
        <Button variant="ghost" className="w-full justify-between p-0">
          <span className="text-sm font-medium">{title}</span>
          <span className="text-xs text-muted-foreground">
            {isAnswerOpen ? "Hide" : "Show"}
          </span>
        </Button>
      </CollapsibleTrigger>
      <CollapsibleContent>
        <div className="text-center space-y-1">
          <p className="text-sm font-medium text-muted-foreground pb-2">
            {subtitle}
          </p>
        </div>

        <div className="space-y-2">
          {gradeItems.map((item, index: number) => (
            <div key={index}>
              <SwipeablePhrase
                ref={(el) => {
                  if (el) {
                    phraseRefs.current.set(index, el);
                  } else {
                    phraseRefs.current.delete(index);
                  }
                }}
                item={item}
                onSwipe={(remembered) => handleGradeSwipe(index, remembered)}
                isSelected={selectedPhraseIndex === index}
                targetLanguage={targetLanguage}
              />
            </div>
          ))}
        </div>
      </CollapsibleContent>
    </Collapsible>
  );
}

export function GramDefinitionDisplay({
  definition,
  breakdown,
  targetLanguage,
}: {
  definition: DefinitionView;
  breakdown?: BreakdownRow[] | null;
  targetLanguage: Language;
}) {
  const hasBreakdown = !!breakdown && breakdown.length > 0;
  const wordText = definition.headword;

  const header = (
    <div className="flex items-start min-w-0 max-w-full">
      {hasBreakdown ? (
        <MorphemeBreakdown
          breakdown={breakdown!}
          targetLanguage={targetLanguage}
          className="text-sm"
        />
      ) : (
        <p className="text-sm font-semibold">
          <TargetLanguageText language={targetLanguage}>
            {wordText}
          </TargetLanguageText>
        </p>
      )}
      <span className="text-sm font-semibold ml-1">:</span>
    </div>
  );

  const body = (
    <div className="space-y-2 grow basis-56 min-w-0">
      {definition.senses.map((sense, index) => (
        <div key={index} className="flex flex-col gap-1">
          <p className="text-sm">
            {sense.meaning}
            {sense.note && (
              <span className="text-muted-foreground"> {sense.note}</span>
            )}
          </p>
          {sense.example && (
            <div className="text-xs text-muted-foreground">
              <p className="italic">
                <TargetLanguageText language={targetLanguage}>
                  "{sense.example.target}"
                </TargetLanguageText>
              </p>
              <p>"{sense.example.native}"</p>
            </div>
          )}
        </div>
      ))}
    </div>
  );

  return (
    // Wrapping row: the definition sits beside the word when it can still get a
    // readable width, and drops to its own full-width line under a wide
    // breakdown instead of being squeezed into a sliver.
    <div className="p-3 px-5 flex flex-wrap items-start gap-x-3 gap-y-1">
      {header}
      {body}
    </div>
  );
}

export function TranslationChallenge({
  sentence,
  initialState,
  onComplete,
  accessToken,
  targetLanguage,
  nativeLanguage,
  autoplayed,
  setAutoplayed,
  deck,
  totalReviewsCompleted,
  pendingReviewScope,
}: SentenceChallengeProps) {
  const [storage] = useState(() => initialState ? undefined : new PendingReview<TranslationState>(
    translation_pending_slot(sentence, pendingReviewScope, get_app_version(), totalReviewsCompleted),
    (payload) => translation_resume(payload as TranslationState).state,
  ));
  const [state, setState] = useState<TranslationState>(() =>
    initialState ?? storage?.load() ?? translation_start(sentence, { targetLanguage, nativeLanguage }),
  );
  const stateRef = useRef(state);
  const view = useMemo(() => translation_view(state), [state]);
  const editing = state.phase.type === "Editing";
  const userTranslation = state.text;
  const correctTranslation = view.correct_translation ?? "";
  const gradeItems = view.grade_section?.items ?? [];
  const canContinue = view.can_continue;
  const tappedDefinitions = view.definitions;
  const verdict: TranslationVerdictData | null = view.verdict ? {
    userTranslation: view.verdict.submission,
    correctTranslation: view.verdict.correct_translation,
    isPerfect: view.verdict.perfect,
    encouragement: view.verdict.encouragement ?? null,
    explanation: view.verdict.explanation ?? null,
    autogradingError: view.verdict.autograding_error ?? null,
    submissionLabel: view.verdict.submission_label,
    correctLabel: view.verdict.correct_label,
  } : null;
  const movieData = useMemo(() => getMovieMetadata(deck, sentence.movie_titles.map(([id]) => id)), [sentence.movie_titles, deck]);
  const [selectedPhraseIndex, setSelectedPhraseIndex] = useState(-1);
  const [showReportModal, setShowReportModal] = useState(false);
  const [hasClip, setHasClip] = useState(false);
  const gradingGenerationRef = useRef(0);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const phraseRefs = useRef<Map<number, SwipeableWordHandle>>(new Map());
  const { bumpBackground } = useBackground();
  const applyStep = useCallback(function apply(step: TranslationStep) {
    stateRef.current = step.state;
    setState(step.state);
    storage?.save(step.state);
    for (const effect of step.effects) {
      switch (effect.type) {
        case "Autograde": {
          bumpBackground(30.0);
          const generation = ++gradingGenerationRef.current;
          void autograde_translation(
            sentence.target_language, effect.submission, sentence.native_translations,
            sentence.target_language_literals, sentence.unique_target_language_phrases,
            accessToken, step.state.course, sentence.gram_definitions_for_lookup,
            new Uint32Array(sentence.literal_gram_indices), sentence.phrase_definitions,
            sentence.primary_expression, sentence.movie_titles,
          ).then(response => {
            if (generation !== gradingGenerationRef.current) return;
            if (response.autograding_error) reportAutogradeFailure("translation", response.autograding_error);
            apply(translation_transition(stateRef.current, { type: "Graded", response }));
          }).catch(error => {
            if (generation !== gradingGenerationRef.current) return;
            const message = error instanceof Error ? error.message : "Failed to grade automatically";
            reportAutogradeFailure("translation", message);
            apply(translation_transition(stateRef.current, { type: "GradingFailed", message }));
          });
          break;
        }
        case "PlaySound":
          playSoundEffect(effect.sound === "AiDoneGrading" ? "aiDoneGrading" : "perfect");
          break;
        case "Complete": {
          const accepted = onComplete(effect.outcome.type === "Perfect" ? { perfect: null } : effect.outcome.grade,
            effect.heteronyms_tapped, effect.submission, effect.completed_at_ms);
          if (accepted) {
            storage?.clear();
            bumpBackground(30.0);
            window.scrollTo({ top: 0, behavior: "smooth" });
          }
          break;
        }
      }
    }
  }, [storage, sentence, accessToken, bumpBackground, onComplete]);
  const send = useCallback((event: TranslationEvent) => {
    if (event.type === "CancelGrading") gradingGenerationRef.current++;
    applyStep(translation_transition(stateRef.current, event));
  }, [applyStep]);
  const resumeStep = useRef(applyStep);
  useEffect(() => {
    const generation = gradingGenerationRef;
    resumeStep.current(translation_resume(stateRef.current));
    return () => { generation.current++; };
  }, []);
  useEffect(() => {
    const timer = setTimeout(() => inputRef.current?.focus(), 100);
    return () => clearTimeout(timer);
  }, [sentence.target_language]);
  const handleWordTap = (index: number) => send({ type: "WordTapped", index });
  const handleCheckAnswer = useCallback(() => send({ type: "Submit", now_ms: Date.now() }), [send]);
  const handleContinue = useCallback(() => send({ type: "Continue" }), [send]);
  const handleGradeSwipe = useCallback((index: number, remembered: boolean) => {
    send({ type: "ItemGraded", item_index: index, grade: remembered ? "Remembered" : "Forgot" });
  }, [send]);

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Enter" && editing) {
        e.preventDefault();
        if (userTranslation.trim()) {
          handleCheckAnswer();
        }
      } else if (e.key === "Enter" && view.verdict) {
        e.preventDefault();
        handleContinue();
      } else if (
        e.key === "ArrowRight" &&
        view.verdict &&
        canContinue
      ) {
        e.preventDefault();
        handleContinue();
        return;
      }

      const hasGradeItems =
        view.verdict &&
        view.grade_section &&
        gradeItems.length > 0;

      if (hasGradeItems) {
        const itemCount = gradeItems.length;

        switch (e.key) {
          case "ArrowUp":
            e.preventDefault();
            setSelectedPhraseIndex((prev) => {
              if (prev <= 0) return itemCount - 1;
              return prev - 1;
            });
            break;

          case "ArrowDown":
            e.preventDefault();
            setSelectedPhraseIndex((prev) => {
              if (prev >= itemCount - 1) return 0;
              return prev + 1;
            });
            break;

          case "ArrowLeft":
            e.preventDefault();
            if (selectedPhraseIndex >= 0 && selectedPhraseIndex < itemCount) {
              const ref = phraseRefs.current.get(selectedPhraseIndex);
              ref?.handleButtonClick(false);
            }
            break;

          case "ArrowRight":
            e.preventDefault();
            if (selectedPhraseIndex >= 0 && selectedPhraseIndex < itemCount) {
              const ref = phraseRefs.current.get(selectedPhraseIndex);
              ref?.handleButtonClick(true);
            }
            break;
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [
    handleContinue,
    selectedPhraseIndex,
    handleGradeSwipe,
    canContinue,
    userTranslation,
    handleCheckAnswer,
    editing,
    view.verdict,
    view.grade_section,
    gradeItems.length,
  ]);

  return (
    <div className="flex flex-col flex-1 justify-between">
      <div>
        <Card animate className="pt-3 pb-3 pl-3 pr-3 relative gap-2">
          {view.badge && (
            <Badge className="absolute -top-2 -left-2 -rotate-12 z-10 shadow-sm text-sm">
              {view.badge}
            </Badge>
          )}
          <div className="space-y-6">
            <div className="text-center">
              <div className="flex items-center justify-between w-full">
                <AudioButton
                  audioRequest={sentence.audio}
                  accessToken={accessToken}
                  autoPlay={!editing && !hasClip}
                  autoplayed={autoplayed}
                  setAutoplayed={setAutoplayed}
                />

                <div className="flex flex-col items-center gap-1">
                  <ChallengeSentence
                    words={view.words}
                    onWordTap={handleWordTap}
                    targetLanguage={targetLanguage}
                  />
                </div>

                <DropdownMenu>
                  <DropdownMenuTrigger asChild>
                    <Button variant="ghost" size="icon" className="h-8 w-8">
                      <MoreVertical className="h-6 w-6 size--xl" />
                    </Button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align="end">
                    <DropdownMenuItem onClick={() => setShowReportModal(true)}>
                      Report an Issue
                    </DropdownMenuItem>
                  </DropdownMenuContent>
                </DropdownMenu>
              </div>
            </div>

            {editing ? (
              <>
                <Textarea
                  ref={inputRef}
                  lang={languageToLangAttr(nativeLanguage)}
                  placeholder={view.placeholder}
                  value={userTranslation}
                  onChange={(e) => send({ type: "TextChanged", text: e.target.value })}
                  className="text-lg min-h-0"
                  rows={1}
                />

                <ProperNounGroups
                  groups={view.proper_nouns}
                  targetLanguage={targetLanguage}
                />
              </>
            ) : (
              <div className="space-y-4 mt-4 animate-feedback-in">
                {view.is_grading ? (
                  <div className="space-y-2">
                    <YourTranslation userTranslation={userTranslation} />
                    <CorrectTranslation sentence={correctTranslation} />
                    <FeedbackSkeleton />
                  </div>
                ) : verdict ? (
                  <>
                    <TranslationVerdict
                      verdict={verdict}
                      targetLanguage={targetLanguage}
                    />

                    {view.grade_section && (
                      <PhraseStatuses
                        gradeItems={gradeItems}
                        phraseRefs={phraseRefs}
                        handleGradeSwipe={handleGradeSwipe}
                        selectedPhraseIndex={selectedPhraseIndex}
                        setSelectedPhraseIndex={setSelectedPhraseIndex}
                        openByDefault={view.grade_section.open_by_default}
                        title={view.grade_section.title}
                        subtitle={view.grade_section.subtitle}
                        targetLanguage={targetLanguage}
                      />
                    )}
                  </>
                ) : null}
              </div>
            )}

            {/* The clip sits under the answer area: it's secondary to the
                sentence being translated, not the headline. */}
            <VideoClipPlayer
              language={targetLanguage}
              text={sentence.target_language}
              accessToken={accessToken}
              autoPlay={!editing}
              autoplayed={autoplayed}
              setAutoplayed={setAutoplayed}
              onAvailabilityChange={setHasClip}
              deck={deck}
            />
          </div>
          {tappedDefinitions.length > 0 && (
            <div className="space-y-2">
              {tappedDefinitions.map((entry, i) => (
                <GramDefinitionDisplay
                  key={i}
                  definition={entry.definition}
                  breakdown={entry.breakdown}
                  targetLanguage={targetLanguage}
                />
              ))}
            </div>
          )}
        </Card>

        {/* Movie posters - hidden after grading */}
        {editing && (
          <MoviePosterGrid movieData={movieData} deck={deck} />
        )}
      </div>

      <div className="sticky bottom-0">
        {editing ? (
          <Button
            onClick={handleCheckAnswer}
            className="w-full mt-4 h-14 text-lg"
            size="lg"
            disabled={!view.can_submit}
          >
            {view.submit_label}
          </Button>
        ) : view.is_grading ? (
          <div className="flex gap-2">
            <Button
              className="flex-1 h-14 text-lg"
              size="lg"
              disabled
            >
              {view.submit_label}
            </Button>
            <Button
              variant="ghost"
              size="icon"
              className="h-14 w-14"
              onClick={() => {
                send({ type: "CancelGrading" });
              }}
            >
              <X className="h-5 w-5" />
            </Button>
          </div>
        ) : (
          <Button
            onClick={handleContinue}
            className="w-full h-14 text-lg"
            size="lg"
            disabled={!canContinue}
          >
            <span className="relative flex items-center justify-center">
              {view.continue_label}
              <span className="absolute left-full ml-2 text-sm text-muted-foreground hide-keyboard-hint-mobile">
                (⏎)
              </span>
            </span>
          </Button>
        )}
      </div>

      <ReportIssueModal
        context={`Sentence challenge: ${JSON.stringify(sentence)}"`}
        open={showReportModal}
        onOpenChange={setShowReportModal}
        targetLanguage={targetLanguage}
      />
    </div>
  );
}
