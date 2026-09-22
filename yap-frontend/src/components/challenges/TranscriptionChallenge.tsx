import { useState, useEffect, useRef, useMemo, useCallback } from "react";
import { PendingReview } from "@/lib/pending-review";
import { getMovieMetadata } from "@/lib/movie-cache";
import { reportAutogradeFailure } from "@/instrument";
import { MoviePosterGrid } from "./MoviePosterGrid";
import {
  autograde_transcription,
  get_app_version,
  type TranscribeComprehensibleSentence,
  type PartGraded,
  transcription_pending_slot,
  transcription_start,
  transcription_resume,
  transcription_transition,
  transcription_view,
  type TranscriptionState,
  type TranscriptionEvent,
  type TranscriptionStep,
  type VerdictView,
  type GradeOptionView,
  get_transcription_review_definitions,
  type WordGrade,
  type Language,
  type Deck,
} from "../../../../yap-frontend-rs/pkg/yap_frontend_rs";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { languageToLangAttr } from "@/lib/utils";
import { Card } from "@/components/ui/card";
import { AudioButton } from "../AudioButton";
import { VideoClipPlayer } from "../VideoClipPlayer";
import { playSoundEffect } from "@/lib/sound-effects";
import { CantListenButton } from "../CantListenButton";
import { AudioErrorBanner } from "../AudioErrorBanner";
import { FeedbackDisplay } from "@/components/FeedbackDisplay";
import { AccentedCharacterKeyboard } from "../AccentedCharacterKeyboard";
import { MobileKeyboardTip } from "../MobileKeyboardTip";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import { useBackground } from "../background-context";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { MoreVertical, X } from "lucide-react";
import { ReportIssueModal } from "./ReportIssueModal";
import { Skeleton } from "@/components/ui/skeleton";
import { InlineTextarea } from "../ui/textarea";
import {
  ProperNounDefinitions,
  GramDefinitionDisplay,
} from "./TranslationChallenge";
import { TargetLanguageText } from "../TargetLanguageText";

interface TranscriptionChallengeProps {
  challenge: TranscribeComprehensibleSentence;
  initialState?: TranscriptionState;
  onComplete: (grade: PartGraded[], completedAtMs: number) => boolean;
  pendingReviewScope: string;
  totalCount: number;
  accessToken: string | undefined;
  onCantListen?: () => void;
  targetLanguage: Language;
  nativeLanguage: Language;
  autoplayed: boolean;
  setAutoplayed: () => void;
  deck: Deck;
  totalReviewsCompleted: bigint;
}

function AutogradeError() {
  return (
    <div
      className={`rounded-lg p-4 border bg-yellow-500/10 border-yellow-500/20`}
    >
      <p
        className={`text-sm font-medium mb-1 text-yellow-600 dark:text-yellow-400`}
      >
        Your submission could not be graded automatically. Please grade the
        words manually below.
      </p>
    </div>
  );
}

function FeedbackSkeleton() {
  return (
    <div className="space-y-4 animate-feedback-in">
      <div className="space-y-3">
        <Skeleton className="h-4 w-3/4" />
        <Skeleton className="h-16 w-full" />
        <Skeleton className="h-4 w-1/2" />
      </div>
    </div>
  );
}

export function TranscriptionChallenge({
  initialState,
  challenge,
  onComplete,
  totalCount,
  accessToken,
  onCantListen,
  targetLanguage,
  nativeLanguage,
  autoplayed,
  setAutoplayed,
  deck,
  totalReviewsCompleted,
  pendingReviewScope,
}: TranscriptionChallengeProps) {
  const [storage] = useState(() => initialState ? undefined : new PendingReview<TranscriptionState>(
    transcription_pending_slot(challenge.parts, pendingReviewScope, get_app_version(), totalReviewsCompleted),
    (payload) => {
      const saved = payload as Omit<TranscriptionState, "inputs"> & { inputs: [number, string][] };
      return transcription_resume({ ...saved, inputs: new Map(saved.inputs) }).state;
    },
    // The bridge exposes BTreeMap as a JS Map, which JSON cannot encode.
    (state) => ({ ...state, inputs: [...state.inputs] }),
  ));
  const [state, setState] = useState<TranscriptionState>(
    () => initialState ?? storage?.load() ?? transcription_start(challenge.parts),
  );
  const stateRef = useRef(state);
  const view = useMemo(() => transcription_view(state), [state]);
  const userInputs = useMemo(
    () => new Map(view.blanks.map((blank) => [blank.index, blank.text])),
    [view.blanks],
  );
  const editing = state.phase.type === "Editing";
  const verdict = view.verdict;
  const [audioError, setAudioError] = useState(false);

  const movieData = useMemo(() => {
    if (!challenge.movie_titles || challenge.movie_titles.length === 0) {
      return [];
    }
    const movieIds = challenge.movie_titles.map(([id]) => id);
    return getMovieMetadata(deck, movieIds);
  }, [challenge.movie_titles, deck]);
  const gradingGenerationRef = useRef(0);
  const [showReportModal, setShowReportModal] = useState(false);
  const [focusedInputIndex, setFocusedInputIndex] = useState<number | null>(
    null,
  );
  const [shiftHeld, setShiftHeld] = useState(false);
  const inputRefs = useRef<(HTMLTextAreaElement | null)[]>([]);
  const { bumpBackground } = useBackground();

  // Find indices of words that should be blanks
  const blankIndices: number[] = useMemo(() => {
    const blankIndices: number[] = [];
    challenge.parts.forEach((item, index) => {
      if (item.type === "AskedToTranscribe") {
        blankIndices.push(index);
      }
    });
    return blankIndices;
  }, [challenge]);

  const wrongGramEntries = useMemo(() => {
    if (state.phase.type !== "Graded") return [];
    return get_transcription_review_definitions(
      challenge,
      state.phase.grade.results,
    );
  }, [state.phase, challenge]);

  const applyStep = useCallback(
    function apply(step: TranscriptionStep) {
      stateRef.current = step.state;
      setState(step.state);
      storage?.save(step.state);
      for (const effect of step.effects) {
        switch (effect.type) {
          case "Autograde": {
            bumpBackground(30.0);
            const generation = ++gradingGenerationRef.current;
            void autograde_transcription(
              effect.submission,
              accessToken,
              {
                targetLanguage,
                nativeLanguage,
              },
              challenge.movie_titles,
            ).then((grade) => {
              if (generation !== gradingGenerationRef.current) return;
              if (grade.autograding_error)
                reportAutogradeFailure(
                  "transcription",
                  grade.autograding_error,
                );
              apply(
                transcription_transition(stateRef.current, {
                  type: "Graded",
                  grade,
                }),
              );
            });
            break;
          }
          case "PlaySound":
            playSoundEffect(
              effect.sound === "AiDoneGrading" ? "aiDoneGrading" : "perfect",
            );
            break;
          case "Complete": {
            const accepted = onComplete(effect.results, effect.completed_at_ms);
            if (accepted) {
              storage?.clear();
              bumpBackground(30.0);
            }
            break;
          }
        }
      }
    },
    [
      storage,
      accessToken,
      bumpBackground,
      challenge,
      nativeLanguage,
      onComplete,
      targetLanguage,
    ],
  );
  const send = useCallback(
    (event: TranscriptionEvent) => {
      if (event.type === "CancelGrading") gradingGenerationRef.current++;
      applyStep(transcription_transition(stateRef.current, event));
    },
    [applyStep],
  );

  // The host keys this component by challenge; resume only its initial snapshot.
  const resumeStep = useRef(applyStep);
  useEffect(() => {
    const generation = gradingGenerationRef;
    resumeStep.current(transcription_resume(stateRef.current));
    return () => {
      generation.current++;
    };
  }, []);

  // Focus the first input on mount, and only on mount. `challenge` is a fresh
  // object every time the host recomputes its view (a clip landing, the
  // readiness tick), so depending on anything derived from it would steal the
  // caret back to the first blank mid-answer.
  const firstBlankIndexRef = useRef(blankIndices[0]);
  useEffect(() => {
    const firstBlankIndex = firstBlankIndexRef.current;
    if (firstBlankIndex === undefined) return;
    const timeout = setTimeout(() => {
      inputRefs.current[firstBlankIndex]?.focus();
    }, 100);
    return () => clearTimeout(timeout);
  }, []);

  // Track shift key state for uppercase accent keyboard
  useEffect(() => {
    const down = (e: KeyboardEvent) => {
      if (e.key === "Shift") setShiftHeld(true);
    };
    const up = (e: KeyboardEvent) => {
      if (e.key === "Shift") setShiftHeld(false);
    };
    window.addEventListener("keydown", down);
    window.addEventListener("keyup", up);
    return () => {
      window.removeEventListener("keydown", down);
      window.removeEventListener("keyup", up);
    };
  }, []);

  // Determine if accent keyboard should show uppercase
  const accentUppercase = useMemo(() => {
    if (shiftHeld) return true;
    // Uppercase when cursor is at position 0 of the first blank and it's the first part
    const firstBlank = blankIndices[0];
    if (firstBlank === undefined) return false;
    const activeIndex = focusedInputIndex ?? firstBlank;
    if (activeIndex !== firstBlank || firstBlank !== 0) return false;
    const value = userInputs.get(activeIndex) || "";
    const input = inputRefs.current[activeIndex];
    const cursorPos = input?.selectionStart ?? 0;
    return cursorPos === 0 && value === "";
  }, [shiftHeld, focusedInputIndex, blankIndices, userInputs]);

  const handleInputChange = (index: number, value: string) => {
    send({ type: "InputChanged", index, text: value });
  };

  const handleCharacterInsert = (char: string) => {
    // Use the last focused input index, or the first blank if none was focused
    const targetIndex =
      focusedInputIndex !== null ? focusedInputIndex : blankIndices[0];

    if (targetIndex !== undefined) {
      const currentValue = userInputs.get(targetIndex) || "";
      const input = inputRefs.current[targetIndex];

      if (input) {
        // Focus the input first to get correct selection
        input.focus();

        const start = input.selectionStart || currentValue.length;
        const end = input.selectionEnd || currentValue.length;
        const newValue =
          currentValue.substring(0, start) + char + currentValue.substring(end);

        handleInputChange(targetIndex, newValue);

        setTimeout(() => {
          if (input) {
            const newPosition = start + char.length;
            input.setSelectionRange(newPosition, newPosition);
            input.focus();
            setFocusedInputIndex(targetIndex);
          }
        }, 0);
      }
    }
  };

  const allBlanksFilledOut = view.can_submit;
  const handleSubmit = useCallback(
    () => send({ type: "Submit", now_ms: Date.now() }),
    [send],
  );
  const handleTranscriptionContinue = useCallback(
    () => send({ type: "Continue" }),
    [send],
  );

  // Global keyboard handler for Enter key
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const activeElement = document.activeElement;
      const isInputFocused =
        activeElement?.tagName === "INPUT" ||
        activeElement?.tagName === "TEXTAREA";

      if (e.key === "Enter") {
        if (isInputFocused) {
          e.preventDefault();

          // Find which input is focused
          const currentIndex = inputRefs.current.findIndex(
            (ref) => ref === activeElement,
          );
          if (currentIndex === -1) return;

          // Find next blank
          const currentBlankPosition = blankIndices.findIndex(
            (index) => index === currentIndex,
          );
          const nextBlankIndex = blankIndices[currentBlankPosition + 1];

          if (nextBlankIndex !== undefined) {
            // Focus next input
            inputRefs.current[nextBlankIndex]?.focus();
          } else if (editing && allBlanksFilledOut) {
            handleSubmit();
          }
        } else if (verdict) {
          e.preventDefault();
          handleTranscriptionContinue();
        }
      } else if (e.key === "ArrowRight" && verdict && !isInputFocused) {
        e.preventDefault();
        handleTranscriptionContinue();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [
    verdict,
    editing,
    handleTranscriptionContinue,
    blankIndices,
    allBlanksFilledOut,
    handleSubmit,
  ]);

  // The sentence with the same words elided as the challenge's blanks, for
  // the video caption before grading — built from the challenge parts (the
  // subtitle cue text may differ cosmetically from the pack sentence, so the
  // parts are the reliable source of what's hidden).
  const maskedSentenceCaption = useMemo(
    () =>
      challenge.parts
        .map((part) =>
          part.type === "Provided"
            ? part.part.word.text + part.part.whitespace
            : part.parts.map((literal) => "____" + literal.whitespace).join(""),
        )
        .join(""),
    [challenge.parts],
  );

  const renderSentenceWithBlanks = () => {
    const askedToTranscribeParts = challenge.parts.filter(
      (part) => part.type === "AskedToTranscribe",
    );
    const isSinglePartTranscription =
      askedToTranscribeParts.length === 1 &&
      challenge.parts.every(
        (part) =>
          part.type === "AskedToTranscribe" ||
          (part.type === "Provided" &&
            part.part.word.word_type?.type !== "Heteronym"),
      );

    return challenge.parts.map((item, index) => {
      if (item.type === "AskedToTranscribe") {
        if (item.parts.length === 0) {
          throw new Error("AskedToTranscribe part has no parts");
        }
        const end_whitespace = item.parts[item.parts.length - 1].whitespace;
        const blank = view.blanks.find((blank) => blank.index === index)!;

        return (
          <span key={index}>
            <InlineTextarea
              ref={(el) => {
                inputRefs.current[index] = el;
              }}
              value={blank.text}
              onChange={(e) => handleInputChange(index, e.target.value)}
              onFocus={() => setFocusedInputIndex(index)}
              onBlur={() => {
                // Keep track of last focused input but allow blur
                // The accent keyboard will refocus when clicked
              }}
              disabled={!blank.editable}
              lang={languageToLangAttr(targetLanguage)}
              autoCorrect="off"
              autoCapitalize={index === 0 ? "sentences" : "off"}
              spellCheck={false}
              className={`inline-block ${
                isSinglePartTranscription ? "min-w-64" : "min-w-32"
              } mx-1 text-center resize-none text-l font-semibold ${getInputClassName(
                index,
              )} border-0 border-b-3 border-dotted`}
              placeholder={view.placeholder}
            />
            <span>{end_whitespace}</span>
          </span>
        );
      } else {
        return (
          <span key={index}>
            <TargetLanguageText language={targetLanguage}>
              {item.part.word.text}
            </TargetLanguageText>
            {item.part.whitespace}
          </span>
        );
      }
    });
  };

  const getInputClassName = (index: number) =>
    ({
      Neutral: "border-muted-foreground/30",
      Perfect: "border-green-500 bg-green-50 dark:bg-green-950",
      PhoneticallyIdentical:
        "border-yellow-500 bg-yellow-50 dark:bg-yellow-950",
      PhoneticallySimilar: "border-orange-500 bg-orange-50 dark:bg-orange-950",
      Wrong: "border-red-500 bg-red-50 dark:bg-red-950",
    })[view.blanks.find((blank) => blank.index === index)!.tint];

  return (
    <div className="flex flex-col flex-1 justify-between">
      <div className="flex flex-col gap-2">
        <Card animate className="pt-3 pb-3 pl-3 pr-3 relative gap-0">
          {challenge.second_chance && (
            <Badge className="absolute -top-2 -left-2 -rotate-12 z-10 shadow-sm text-sm">
              Second Chance!
            </Badge>
          )}
          {/* Dropdown menu for options */}
          <div className="absolute top-2 right-2">
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button variant="ghost" size="icon" className="h-8 w-8">
                  <MoreVertical className="h-4 w-4" />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end">
                <DropdownMenuItem onClick={() => setShowReportModal(true)}>
                  Report an Issue
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>

          <div className="space-y-6">
            {/* Audio + sentence with blanks, grouped so they read as one unit */}
            <div>
              <div className="flex flex-col items-center gap-2">
                <AudioButton
                  audioRequest={challenge.audio}
                  accessToken={accessToken}
                  autoPlay={true}
                  autoplayed={autoplayed}
                  setAutoplayed={setAutoplayed}
                  playPreAudio={true}
                  onError={() => setAudioError(true)}
                  onSuccess={() => setAudioError(false)}
                  visualizer
                />
                <p className="text-sm text-muted-foreground">
                  {view.instructions}
                </p>
              </div>
              <div className="text-center pt-4">
                <div className="text-2xl font-semibold leading-relaxed">
                  {renderSentenceWithBlanks()}
                </div>
              </div>
            </div>

            {/* The clip sits under the answer area as secondary media.
                Watchable before grading — hearing the line is the exercise —
                but its caption elides the blanked words until grading, so
                the subtitle can't give the answer away. */}
            <VideoClipPlayer
              language={targetLanguage}
              text={challenge.target_language}
              accessToken={accessToken}
              deck={deck}
              renderSentenceCue={(text) =>
                editing ? (
                  <TargetLanguageText language={targetLanguage}>
                    {maskedSentenceCaption}
                  </TargetLanguageText>
                ) : (
                  <TargetLanguageText language={targetLanguage}>
                    {text}
                  </TargetLanguageText>
                )
              }
            />

            {editing && (
              <ProperNounDefinitions
                definitions={challenge.proper_noun_definitions}
                targetLanguage={targetLanguage}
              />
            )}

            {/* Result feedback */}
            {!editing && (
              <div className="space-y-2 animate-feedback-in">
                {/* Show correct answer immediately when grading starts */}
                <div className="rounded-lg p-4 border bg-green-500/10 border-green-500/20">
                  <p className="text-sm font-medium mb-1 text-green-600 dark:text-green-400">
                    {verdict?.correct_label ?? "Correct sentence:"}
                  </p>
                  <p className="text-lg font-medium">
                    <TargetLanguageText language={targetLanguage}>
                      {challenge.target_language}
                    </TargetLanguageText>
                  </p>
                </div>

                {/* Show skeleton while grading */}
                {view.is_grading && <FeedbackSkeleton />}

                {/* Only show these when grading is complete */}
                {verdict && (
                  <>
                    {"autograding_error" in verdict &&
                      verdict.autograding_error && <AutogradeError />}

                    <WordGrades
                      verdict={verdict}
                      gradeOptions={view.grade_options}
                      setGrade={(part_index, word_index, grade) =>
                        send({
                          type: "WordGradeChanged",
                          part_index,
                          word_index,
                          grade,
                        })
                      }
                      open_by_default={
                        "autograding_error" in verdict &&
                        verdict.autograding_error !== undefined
                      }
                      targetLanguage={targetLanguage}
                    />

                    <FeedbackDisplay
                      encouragement={verdict.encouragement}
                      explanation={verdict.explanation}
                      perfect={verdict.perfect}
                      targetLanguage={targetLanguage}
                    />

                    {Array.isArray(verdict.compare) &&
                      verdict.compare.length > 0 &&
                      (() => {
                        const words = verdict.compare;

                        const ttsText = words.map((w) => `${w};`).join(" ");

                        return (
                          <div className="rounded-lg p-4 border">
                            <div className="flex flex-row items-center gap-3">
                              <p className="text-sm font-medium">Listen:</p>
                              <AudioButton
                                audioRequest={{
                                  request: {
                                    text: ttsText,
                                    language: targetLanguage,
                                    speed: 0.8,
                                  },
                                  provider: "Google",
                                }}
                                accessToken={accessToken}
                                size="icon"
                                variant="ghost"
                                temp
                              />
                              <div className="flex flex-row flex-wrap justify-around items-center gap-3">
                                {words.map((item, idx) => (
                                  <span key={idx} className="font-medium">
                                    <TargetLanguageText
                                      language={targetLanguage}
                                    >
                                      {item}
                                    </TargetLanguageText>
                                    {idx < words.length - 1 && ","}
                                  </span>
                                ))}
                              </div>
                            </div>
                          </div>
                        );
                      })()}

                    <div
                      className="rounded-lg p-4 border cursor-pointer select-none"
                      onClick={() => send({ type: "TranslationToggled" })}
                    >
                      <p className="text-sm font-medium mb-1">
                        English translation (click to reveal):
                      </p>
                      <p
                        className={`text-lg font-medium transition-all duration-100 ${
                          verdict.translation_revealed ? "" : "blur-sm"
                        }`}
                      >
                        {challenge.native_language}
                      </p>
                    </div>

                    {wrongGramEntries.length > 0 && (
                      <div className="space-y-2">
                        {wrongGramEntries.map((entry, i) => (
                          <GramDefinitionDisplay
                            key={i}
                            definition={entry.definition}
                            breakdown={entry.breakdown}
                            targetLanguage={targetLanguage}
                          />
                        ))}
                      </div>
                    )}
                  </>
                )}
              </div>
            )}
          </div>
        </Card>

        {audioError && onCantListen && editing && (
          <AudioErrorBanner onSkip={onCantListen} />
        )}

        {/* Accented character keyboard - show when not graded, language supports it, and not on small screens */}
        {editing &&
          (targetLanguage === "French" ||
            targetLanguage === "Spanish" ||
            targetLanguage === "German") && (
            <AccentedCharacterKeyboard
              onCharacterInsert={handleCharacterInsert}
              language={targetLanguage}
              uppercase={accentUppercase}
              className="hidden md:flex mt-3 p-3 border rounded-lg bg-muted/30"
            />
          )}

        {/* Mobile keyboard tip - show on small screens when conditions are met */}
        {editing && totalCount < 60 && (
          <MobileKeyboardTip language={targetLanguage} />
        )}

        {/* Movie posters - hidden after grading */}
        {editing && <MoviePosterGrid movieData={movieData} deck={deck} />}
      </div>

      <div className="mt-4 flex flex-col gap-2 sticky bottom-0">
        {onCantListen && editing && <CantListenButton onClick={onCantListen} />}

        <div>
          {view.is_grading ? (
            <div className="flex gap-2">
              <Button className="flex-1 h-14 text-lg" size="lg" disabled>
                AI is grading...
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
              onClick={verdict ? handleTranscriptionContinue : handleSubmit}
              disabled={editing ? !view.can_submit : !view.can_continue}
              className="w-full h-14 text-lg"
              size="lg"
            >
              {editing ? (
                <span className="relative flex items-center justify-center">
                  {view.submit_label}
                  <span className="absolute left-full ml-2 text-sm text-muted-foreground hide-keyboard-hint-mobile">
                    (⏎)
                  </span>
                </span>
              ) : (
                <span className="relative flex items-center justify-center">
                  {verdict?.continue_label}
                  <span className="absolute left-full ml-2 text-sm text-muted-foreground hide-keyboard-hint-mobile">
                    (⏎)
                  </span>
                </span>
              )}
            </Button>
          )}
        </div>
      </div>

      <ReportIssueModal
        context={`Transcription challenge: ${JSON.stringify(challenge)}`}
        open={showReportModal}
        onOpenChange={setShowReportModal}
        targetLanguage={targetLanguage}
      />
    </div>
  );
}

interface WordGradesProps {
  verdict: VerdictView;
  gradeOptions: GradeOptionView[];
  setGrade: (partIndex: number, wordIndex: number, grade: WordGrade) => void;
  open_by_default: boolean;
  targetLanguage: Language;
}

function WordGrades({
  verdict,
  gradeOptions,
  setGrade,
  open_by_default,
  targetLanguage,
}: WordGradesProps) {
  const [isOpen, setIsOpen] = useState(open_by_default);
  if (verdict.word_grades.length === 0) return null;
  return (
    <Collapsible open={isOpen} onOpenChange={setIsOpen}>
      <CollapsibleTrigger asChild>
        <Button variant="ghost" className="w-full justify-between p-0">
          <span className="text-sm font-medium">Grade Words Manually</span>
          <span className="text-xs text-muted-foreground">
            {isOpen ? "Hide" : "Show"}
          </span>
        </Button>
      </CollapsibleTrigger>
      <CollapsibleContent>
        <div className="pt-3 space-y-3">
          <div className="text-sm text-muted-foreground">
            {verdict.submission_label}{" "}
            <TargetLanguageText language={targetLanguage}>
              {verdict.submission_text}
            </TargetLanguageText>
          </div>
          {verdict.word_grades.map((word) => (
            <div
              key={`${word.part_index}-${word.word_index}`}
              className="flex items-center gap-3 p-2 rounded-lg bg-muted/30"
            >
              <div className="flex-1 font-medium">
                <TargetLanguageText language={targetLanguage}>
                  {word.heard}
                </TargetLanguageText>
              </div>
              <Select
                value={String(word.selected)}
                onValueChange={(value) =>
                  setGrade(
                    word.part_index,
                    word.word_index,
                    gradeOptions[Number(value)].grade,
                  )
                }
              >
                <SelectTrigger className="w-[200px]">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {gradeOptions.map((option, index) => (
                    <SelectItem key={index} value={String(index)}>
                      {option.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          ))}
        </div>
      </CollapsibleContent>
    </Collapsible>
  );
}
