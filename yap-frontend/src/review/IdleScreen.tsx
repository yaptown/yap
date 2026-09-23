import { Button } from "@/components/ui/button";
import { EngagementPrompts } from "@/review/engagement-prompts";
import type {
  EmphasizedText,
  IdleScreenView,
  IdleView,
  DeckEvent,
  Deck,
  Language,
  MovieMetadataBasic,
} from "../../../yap-frontend-rs/pkg";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  ChevronDown,
  ChevronLeft,
  ChevronRight,
  Headphones,
  LoaderCircle,
  Sparkles,
} from "lucide-react";
import { Card } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import {
  Tooltip,
  TooltipTrigger,
  TooltipContent,
} from "@/components/ui/tooltip";
import { memo, useCallback, useEffect, useState } from "react";
import { Poster } from "@/browse/Poster";
import { SwitchCurriculumButton } from "@/review/SwitchCurriculumButton";
import { TargetLanguageText } from "../components/TargetLanguageText";
import { ReviewPlanCard } from "./ladder/ReviewPlanScreen";
import { WeekProgressStrip } from "./WeekProgressStrip";
import { sentenceListSelectionToSentenceList, type SentenceList } from "@/browse/useSentenceList";
import { useNavigate } from "react-router-dom";

export interface MovieWithMetadata extends MovieMetadataBasic {
  percent_known: number;
  all_available_learned: boolean;
  cards_to_next_milestone: number | null | undefined;
}

interface IdleScreenProps {
  view: IdleScreenView;
  showEngagementPrompts: boolean;
  addEvent: (event: DeckEvent) => void;
  undoRestrictions: () => void;
  deck: Deck;
  setSentenceList: (sentenceList: SentenceList) => void;
  commitSentenceList: (event: DeckEvent) => void;
}

export const IdleScreen = memo(function IdleScreen(props: IdleScreenProps) {
  const { view, deck, addEvent } = props;
  const [showReleasePlan, setShowReleasePlan] = useState(false);
  const plan = view.type === "ReviewPlanOffer" ? view : view.type === "StudyPlanComplete" ? view.plan : undefined;
  useEffect(() => {
    if (!plan) return;
    const key = (event: KeyboardEvent) => {
      if ((event.target as HTMLElement).closest("input, textarea, select, button, a")) return;
      if (event.code !== "Space" && event.code !== "Enter") return;
      event.preventDefault();
      if (view.type === "StudyPlanComplete" && !showReleasePlan) setShowReleasePlan(true);
      else addEvent(plan.event);
    };
    window.addEventListener("keydown", key);
    return () => window.removeEventListener("keydown", key);
  }, [plan, view.type, showReleasePlan, addEvent]);
  switch (view.type) {
    case "AudioPending": return (
      <div className="flex flex-col flex-1 gap-4 pt-4">
        <div className="flex flex-col gap-2 text-center">
          <p className="text-2xl font-bold">Just a moment…</p>
          <p className="text-muted-foreground">Downloading the audio for your next challenge.</p>
          {!view.online && <p>Reconnect to download audio.</p>}
        </div>
        <div className="flex justify-center py-4"><LoaderCircle className="h-8 w-8 animate-spin text-muted-foreground" /></div>
        <WeekProgressStrip week={view.week} className="mt-auto mb-2" />
      </div>
    );
    case "StudyPlanComplete":
      if (!showReleasePlan) return (
        <div className="flex flex-col flex-1 gap-4 pt-4">
          <div className="flex flex-col gap-2 text-center">
            <p className="text-2xl font-bold">{view.title}</p>
            {view.next_review && <NextReviewLine line={view.next_review} targetLanguage={view.plan.target_language} />}
          </div>
          <div className="flex justify-center"><Button onClick={() => setShowReleasePlan(true)} size="lg" variant="outline">Study more</Button></div>
          <WeekProgressStrip week={view.plan.week} className="mt-auto mb-2" />
        </div>
      );
      return <ReviewPlanCard title="Today's review plan:" cards={view.plan.cards} buttonLabel="Let's go!" onCommit={() => addEvent(view.plan.event)} week={view.plan.week} targetLanguage={view.plan.target_language} />;
    case "ReviewPlanOffer": return <ReviewPlanCard title="Today's review plan:" cards={view.cards} buttonLabel="Let's go!" onCommit={() => addEvent(view.event)} week={view.week} targetLanguage={view.target_language} />;
    case "Idle": return <IdleContent {...props} view={view} deck={deck} />;
  }
});

function IdleContent({ view, showEngagementPrompts, addEvent, undoRestrictions, deck, setSentenceList, commitSentenceList }: Omit<IdleScreenProps, "view"> & { view: IdleView }) {
  const navigate = useNavigate();
  const [pimsleurAcknowledged, setPimsleurAcknowledged] = useState(() => localStorage.getItem("yap-pimsleur-acknowledged") === "true");
  const targetLanguage = view.target_language;
  const info = view.info;
  const manualAddOptions = view.manual_add_options;
  const addSmartCards = useCallback(() => { if (info.smart_add_event) addEvent(info.smart_add_event); }, [info.smart_add_event, addEvent]);
  useEffect(() => {
    const key = (event: KeyboardEvent) => {
      if ((event.target as HTMLElement).closest("input, textarea, select, button, a")) return;
      if ((event.code === "Space" || event.code === "Enter") && info.smart_add_event) {
        event.preventDefault(); addSmartCards();
      }
    };
    window.addEventListener("keydown", key);
    return () => window.removeEventListener("keydown", key);
  }, [info.smart_add_event, addSmartCards]);
  const navigation = view.navigation;
  const categories = navigation.categories;
  const effectiveIndex = navigation.selected_index;
  const effectiveSentenceList = sentenceListSelectionToSentenceList(navigation.selection);

  const canGoLeft = effectiveIndex > 0;
  const canGoRight = effectiveIndex < categories.length - 1;

  const navigateSentenceList = (direction: "left" | "right") => {
    const nextIndex =
      direction === "left" ? effectiveIndex - 1 : effectiveIndex + 1;
    if (nextIndex >= 0 && nextIndex < categories.length) {
      setSentenceList(sentenceListSelectionToSentenceList(view.sentence_list_options[nextIndex].selection));
    }
  };

  // Sentence list progress info
  const { percent_known: sentenceListPercentKnown, all_available_learned: sentenceListDone } = view.progress;
  const sentenceListLabel = view.sentence_list_label;

  const sentenceListImage = (() => {
    switch (effectiveSentenceList.type) {
      case "essential":
        return { type: "url" as const, url: "/essential-course.webp" };
      case "movie":
        return { type: "movie" as const, movieId: effectiveSentenceList.movieId };
      case "pimsleur":
        return null;
    }
  })();

  return (
    <div className="flex flex-col flex-1 gap-4">
      <div className="text-center py-4">
        <div className="flex flex-col gap-2">
          <p className="text-2xl font-bold">
            {view.title}
          </p>
          {view.body ? <p className="text-muted-foreground">{view.body}</p> : view.next_review && <NextReviewLine line={view.next_review} targetLanguage={targetLanguage} />}
          {view.banned_notice && <><p className="text-muted-foreground">{view.banned_notice}</p><Button variant="outline" onClick={undoRestrictions}>Undo restrictions</Button></>}

        </div>
      </div>

      {view.smart_add_label && (
        <div className="flex justify-center">
          <Button
            onClick={addSmartCards}
            variant="default"
            size="lg"
            className="group relative overflow-hidden transition-all hover:scale-105 hover:shadow-lg"
          >
            <span className="absolute inset-0 bg-gradient-to-r from-transparent via-white/20 to-transparent translate-x-[-200%] group-hover:translate-x-[200%] transition-transform duration-1000"></span>
            <Sparkles className="h-5 w-5 mr-2 animate-pulse" />
            {view.smart_add_label}
          </Button>
        </div>
      )}

      {view.show_sentence_list && (
        <Card className="overflow-hidden px-2 py-4 gap-2" animate>
          <p className="text-lg font-semibold px-4 sm:px-8 text-center">
            {view.curriculum_headline.before}
            <br />
            <span className="uppercase font-bold">{view.curriculum_headline.emphasis}{view.curriculum_headline.after}</span>
          </p>
          <div className="flex items-center justify-between gap-0">
            <button
              onClick={() => navigateSentenceList("left")}
              className={`hidden sm:flex p-2 self-stretch items-center transition-colors ${canGoLeft ? "text-foreground/60 hover:text-foreground hover:bg-muted/50" : "text-transparent cursor-default"}`}
              disabled={!canGoLeft}
              aria-label="Previous sentence list"
            >
              <ChevronLeft className="h-6 w-6" />
            </button>

            <div className="flex-1 flex flex-col sm:flex-row items-center gap-4">
              {effectiveSentenceList.type === "pimsleur" && !pimsleurAcknowledged ? (
                <div className="flex-1 flex flex-col items-center gap-3 py-4 px-2 text-center">
                  <Headphones className="h-8 w-8 text-muted-foreground" />
                  <p className="text-sm text-muted-foreground">
                    Yap has word lists for Pimsleur, but is not affiliated with
                    Pimsleur in any way.
                  </p>
                  <Button
                    variant="default"
                    onClick={() => {
                      localStorage.setItem("yap-pimsleur-acknowledged", "true");
                      setPimsleurAcknowledged(true);
                    }}
                  >
                    I understand
                  </Button>
                </div>
              ) : (
                <>
                  <div
                    onClick={() => navigate("/goals")}
                    className="hidden sm:block sm:order-first w-24 h-36 flex-shrink-0 rounded-lg border border-border/50 overflow-hidden cursor-pointer hover:scale-105 transition-all"
                  >
                    {sentenceListImage?.type === "url" ? (
                      <img
                        src={sentenceListImage.url}
                        alt={sentenceListLabel}
                        className={`w-full h-full object-cover opacity-90 saturate-70 dark:opacity-70 dark:saturate-80 hover:opacity-100 hover:saturate-100 transition-all ${effectiveSentenceList.type === "essential" ? "dark:invert dark:hue-rotate-180" : ""}`}
                      />
                    ) : sentenceListImage?.type === "movie" ? (
                      <Poster
                        movieId={sentenceListImage.movieId}
                        deck={deck}
                        alt={sentenceListLabel}
                      />
                    ) : (
                      <div className="w-full h-full bg-muted flex items-center justify-center">
                        <Headphones className="h-8 w-8 text-muted-foreground" />
                      </div>
                    )}
                  </div>
                  <div className="order-1 sm:order-last flex-1 flex flex-col items-center sm:items-start gap-3 min-w-0 w-full sm:w-auto">
                    {view.next_sentence_list_label && view.next_sentence_list ? (
                      <Button
                        onClick={() => setSentenceList(sentenceListSelectionToSentenceList(view.next_sentence_list!))}
                        variant="default"
                        size="lg"
                        className="group relative overflow-hidden transition-all hover:scale-105 hover:shadow-lg"
                      >
                        <ChevronRight className="h-5 w-5 mr-2" />
                        {view.next_sentence_list_label}
                      </Button>
                    ) : view.all_learned_note ? (
                      <p className="text-sm">{view.all_learned_note}</p>
                    ) : view.curriculum_learn_label ? (
                      <div className="flex">
                        <Tooltip>
                          <TooltipTrigger asChild>
                            <Button
                              onClick={addSmartCards}
                              variant="default"
                              size="lg"
                              className="group relative overflow-hidden transition-all hover:scale-105 hover:shadow-lg whitespace-normal h-auto min-h-10 rounded-r-none"
                            >
                              <span className="absolute inset-0 bg-gradient-to-r from-transparent via-white/20 to-transparent translate-x-[-200%] group-hover:translate-x-[200%] transition-transform duration-1000"></span>
                              <Sparkles className="h-5 w-5 mr-2 animate-pulse" />
                              {view.curriculum_learn_label}
                            </Button>
                          </TooltipTrigger>
                          {info.preview.length > 0 && (
                            <TooltipContent>
                              {info.preview.join(", ")}
                            </TooltipContent>
                          )}
                        </Tooltip>
                        <DropdownMenu>
                          <DropdownMenuTrigger asChild>
                            <Button
                              variant="default"
                              size="lg"
                              className="rounded-l-none border-l border-l-primary-foreground/20 px-2"
                              aria-label={view.manual_add_heading}
                            >
                              <ChevronDown className="h-4 w-4" />
                            </Button>
                          </DropdownMenuTrigger>
                          <DropdownMenuContent align="end">
                            {manualAddOptions.map((option) => (
                              <DropdownMenuItem
                                key={option.card_type}
                                onClick={() => addEvent(option.event)}
                                className="cursor-pointer"
                              >
                                <Sparkles className="h-4 w-4 mr-2" />
                                {option.label}
                              </DropdownMenuItem>
                            ))}
                          </DropdownMenuContent>
                        </DropdownMenu>
                      </div>
                    ) : (
                      <p className="text-sm">
                        You've learned all available words!
                      </p>
                    )}

                    <Progress
                      value={sentenceListPercentKnown}
                      projectedValue={
                        sentenceListDone
                          ? undefined
                          : info.percent_known_after
                      }
                      showPercentage
                      label={sentenceListDone ? "Done!" : undefined}
                      className="h-6"
                    />

                    {view.level_note && (
                      <p className="text-xs text-muted-foreground text-center sm:text-left">
                        {view.level_note}
                      </p>
                    )}

                    <button
                      onClick={() => navigate("/goals")}
                      className="text-xs text-foreground/60 hover:text-foreground underline underline-offset-2 transition-colors text-left"
                    >
                      change sentence list
                    </button>

                    {categories.length > 1 && (
                      <div className="flex sm:hidden items-center justify-between w-full">
                        <button
                          onClick={() => navigateSentenceList("left")}
                          className={`p-2 transition-colors ${canGoLeft ? "text-foreground/60 hover:text-foreground" : "text-transparent cursor-default"}`}
                          disabled={!canGoLeft}
                          aria-label="Previous sentence list"
                        >
                          <ChevronLeft className="h-6 w-6" />
                        </button>
                        <button
                          onClick={() => navigateSentenceList("right")}
                          className={`p-2 transition-colors ${canGoRight ? "text-foreground/60 hover:text-foreground" : "text-transparent cursor-default"}`}
                          disabled={!canGoRight}
                          aria-label="Next sentence list"
                        >
                          <ChevronRight className="h-6 w-6" />
                        </button>
                      </div>
                    )}
                  </div>
                </>
              )}
            </div>

            <button
              onClick={() => navigateSentenceList("right")}
              className={`hidden sm:flex p-2 self-stretch items-center transition-colors ${canGoRight ? "text-foreground/60 hover:text-foreground hover:bg-muted/50" : "text-transparent cursor-default"}`}
              disabled={!canGoRight}
              aria-label="Next sentence list"
            >
              <ChevronRight className="h-6 w-6" />
            </button>
          </div>
        </Card>
      )}

      <SwitchCurriculumButton commit={view.switch_curriculum} onCommit={commitSentenceList} />

      {showEngagementPrompts && <EngagementPrompts language={targetLanguage} />}

      {view.show_sentence_list && (
        <WeekProgressStrip week={view.week} className="mt-auto mb-2" />
      )}
    </div>
  );
}

/// Rust builds the sentence; the emphasized run is the target-language word.
function NextReviewLine({
  line,
  targetLanguage,
}: {
  line: EmphasizedText;
  targetLanguage: Language;
}) {
  return (
    <p className="text-muted-foreground">
      {line.before}
      {line.emphasis && (
        <span className="font-semibold">
          <TargetLanguageText language={targetLanguage}>
            {line.emphasis}
          </TargetLanguageText>
        </span>
      )}
      {line.after}
    </p>
  );
}
