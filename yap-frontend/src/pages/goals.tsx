import { useState } from "react";
import { useNavigate } from "react-router-dom";
import type { UserInfo } from "@/App";
import type {
  Deck as DeckType,
  GoalsScreenView,
} from "../../../yap-frontend-rs/pkg";
import { DeckPage } from "@/components/DeckPage";
import { TopPageLayout } from "@/components/TopPageLayout";
import { GoalProgress } from "@/components/GoalProgress";
import { DailyGoalEditor } from "@/components/DailyGoalEditor";
import { Movies } from "@/components/Movies";
import { Card } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import {
  Collapsible,
  CollapsibleTrigger,
  CollapsibleContent,
} from "@/components/ui/collapsible";
import { Check, ChevronDown, Headphones } from "lucide-react";
import { languageToIso6391 } from "@/lib/utils";
import { getMovieMetadata } from "@/lib/movie-cache";
import {
  sentenceListSelectionToSentenceList,
  sentenceListToSelection,
  type SentenceList,
} from "@/hooks/useSentenceList";
import { useStudyScreenInputs } from "@/hooks/useStudyScreenInputs";
import { useWeapon } from "@/weapon";

export function GoalsPage() {
  return <DeckPage>{(props) => <GoalsScreen {...props} />}</DeckPage>;
}

export function GoalsScreen({
  deck,
  userInfo,
  view: injectedView,
}: {
  view?: GoalsScreenView;
  deck: DeckType;
  userInfo: UserInfo | undefined;
}) {
  const navigate = useNavigate();
  const weapon = useWeapon();
  const inputs = useStudyScreenInputs(!injectedView);
  const view =
    injectedView ?? deck.goals_screen_view(inputs.banned, inputs.sentence_list);
  const curriculum = view.curriculum;
  const sentenceList = sentenceListSelectionToSentenceList(
    curriculum.navigation.selection,
  );
  const [pimsleurAcknowledged, setPimsleurAcknowledged] = useState(
    () => localStorage.getItem("yap-pimsleur-acknowledged") === "true",
  );
  const addEvent = (event: Parameters<typeof weapon.add_deck_event>[0]) => {
    if (!injectedView) weapon.add_deck_event(event);
  };
  const setSentenceList = (sl: SentenceList) =>
    addEvent(deck.change_sentence_list(sentenceListToSelection(sl)));
  // These optional lists are not part of GoalsScreenView; never substitute live
  // deck data into a capture. The captured curriculum card still renders above.
  const movieStats = injectedView ? [] : deck.get_movie_stats();
  const metadata = new Map(
    getMovieMetadata(
      deck,
      movieStats.map((movie) => movie.id),
    ).map((movie) => [movie.id, movie]),
  );
  const moviesWithMetadata = movieStats.flatMap((stat) => {
    const movie = metadata.get(stat.id);
    return movie ? [{ ...movie, ...stat }] : [];
  });
  const pimsleurStats =
    !injectedView && curriculum.has_pimsleur ? deck.get_pimsleur_stats() : [];

  return (
    <TopPageLayout
      userInfo={userInfo}
      headerProps={{
        backButton: { label: "Home", onBack: () => navigate("/home") },
      }}
    >
      <main className="flex flex-col gap-6 py-4">
        <h1 className="text-2xl font-bold">{view.title}</h1>
        <Card className="p-5 gap-3">
          <GoalProgress goal={view.goal} />
        </Card>
        <Card className="p-5 gap-4">
          <div className="flex flex-col gap-1">
            <h2 className="text-lg font-semibold">{view.daily_goal_title}</h2>
            <p className="text-sm text-muted-foreground">
              {view.daily_goal_label}
            </p>
          </div>
          <DailyGoalEditor
            key={view.daily_goal}
            target={view.daily_goal}
            options={view.daily_goal_options}
            addEvent={addEvent}
          />
        </Card>
        <section className="flex flex-col gap-4">
          <h2 className="text-xl font-semibold">{curriculum.title}</h2>
          <Card className="p-5 gap-4">
            <Tabs
              value={
                curriculum.sentence_list_options[
                  curriculum.navigation.selected_index
                ].category
              }
              onValueChange={(category) => {
                const option = curriculum.sentence_list_options.find(
                  (option) => option.category === category,
                );
                if (option) addEvent(option.event);
              }}
              className="gap-4"
            >
              <TabsList className="w-full" aria-label={curriculum.title}>
                {curriculum.sentence_list_options.map((option) => (
                  <TabsTrigger
                    key={option.category}
                    value={option.category}
                    className="data-[state=active]:bg-primary data-[state=active]:text-primary-foreground dark:data-[state=active]:bg-primary dark:data-[state=active]:text-primary-foreground"
                  >
                    {option.label}
                  </TabsTrigger>
                ))}
              </TabsList>
              <TabsContent value="essential" className="space-y-4">
                <h3 className="font-semibold">
                  {curriculum.sentence_list_label}
                </h3>
                <Progress
                  className="h-6"
                  value={curriculum.progress.percent_known}
                  showPercentage
                  aria-label={curriculum.sentence_list_label}
                />
                {curriculum.next_sentence_list_event && (
                  <Button
                    variant="outline"
                    onClick={() =>
                      addEvent(curriculum.next_sentence_list_event!)
                    }
                  >
                    {curriculum.next_sentence_list?.type === "Movie"
                      ? "Next movie"
                      : "Next lesson"}
                  </Button>
                )}
              </TabsContent>
              <TabsContent value="movie">
                {!injectedView && curriculum.has_movies && (
                  <Movies
                    moviesWithMetadata={moviesWithMetadata}
                    targetLanguageIso={languageToIso6391(view.target_language)}
                    deck={deck}
                    selectedMovieId={
                      sentenceList.type === "movie"
                        ? sentenceList.movieId
                        : undefined
                    }
                    onSelectMovie={(id) =>
                      setSentenceList({ type: "movie", movieId: id })
                    }
                  />
                )}
              </TabsContent>
              <TabsContent value="pimsleur" className="space-y-4">
                {/* Pimsleur sentence lists */}
                {pimsleurStats.length > 0 && (
                  <>
                    <h3 className="text-lg font-semibold">Pimsleur Lessons</h3>
                    {!pimsleurAcknowledged ? (
                      <div className="flex flex-col items-center gap-3 py-4 text-center">
                        <p className="text-sm text-muted-foreground">
                          Yap has word lists for Pimsleur, but is not affiliated
                          with Pimsleur in any way.
                        </p>
                        <Button
                          variant="default"
                          onClick={() => {
                            localStorage.setItem(
                              "yap-pimsleur-acknowledged",
                              "true",
                            );
                            setPimsleurAcknowledged(true);
                          }}
                        >
                          I understand
                        </Button>
                      </div>
                    ) : (
                      <>
                        <p className="text-sm text-muted-foreground">
                          Focus on vocabulary from a specific Pimsleur lesson.
                        </p>

                        {(() => {
                          const levels = [
                            ...new Set(pimsleurStats.map((l) => l.level)),
                          ].sort((a, b) => a - b);
                          return levels.map((level) => {
                            const units = pimsleurStats.filter(
                              (l) => l.level === level,
                            );
                            return (
                              <Collapsible
                                key={level}
                                className="flex flex-col gap-2"
                              >
                                <CollapsibleTrigger className="flex items-center gap-2 w-full group">
                                  <h4 className="text-sm font-semibold">
                                    Level {level}
                                  </h4>
                                  <ChevronDown className="h-3 w-3 text-muted-foreground transition-transform group-data-[state=open]:rotate-180" />
                                </CollapsibleTrigger>
                                <CollapsibleContent>
                                  <div className="space-y-2">
                                    {units.map((lesson) => {
                                      const isSelected =
                                        sentenceList.type === "pimsleur" &&
                                        sentenceList.level === lesson.level &&
                                        sentenceList.lesson === lesson.lesson;

                                      return (
                                        <SentenceListCard
                                          key={`pimsleur-${lesson.level}-${lesson.lesson}`}
                                          selected={isSelected}
                                          onClick={() =>
                                            setSentenceList({
                                              type: "pimsleur",
                                              level: lesson.level,
                                              lesson: lesson.lesson,
                                            })
                                          }
                                          title={`Lesson ${lesson.lesson}`}
                                          percentKnown={lesson.percent_known}
                                          done={lesson.all_available_learned}
                                        />
                                      );
                                    })}
                                  </div>
                                </CollapsibleContent>
                              </Collapsible>
                            );
                          });
                        })()}
                      </>
                    )}
                  </>
                )}
              </TabsContent>
            </Tabs>
          </Card>
        </section>
      </main>
    </TopPageLayout>
  );
}

function SentenceListCard({
  selected,
  onClick,
  title,
  percentKnown,
  done,
}: {
  selected: boolean;
  onClick: () => void;
  title: string;
  percentKnown: number;
  done: boolean;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      aria-pressed={selected}
      className="w-full text-left rounded-xl focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
    >
      <Card
        className={`p-3 flex-row items-center gap-3 ${selected ? "ring-2 ring-primary" : "hover:bg-muted/50"}`}
      >
        <Headphones className="h-5 w-5 shrink-0 text-muted-foreground" />
        <div className="flex flex-1 min-w-0 flex-col gap-2">
          <div className="flex items-center gap-2">
            <span className="font-semibold text-sm">{title}</span>
            {selected && <Check className="h-4 w-4 text-primary shrink-0" />}
          </div>
          <div className="flex items-center gap-2">
            <Progress value={percentKnown} aria-label={title} />
            <span className="text-xs tabular-nums text-muted-foreground">
              {done ? "Done!" : `${Math.floor(percentKnown)}%`}
            </span>
          </div>
        </div>
      </Card>
    </button>
  );
}
