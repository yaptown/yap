import { useState } from "react";
import { Check, ChevronDown, Headphones } from "lucide-react";
import type {
  Deck,
  DeckEvent,
  SentenceListCategory,
  SentenceListSelection,
} from "../../../yap-frontend-rs/pkg";
import {
  get_language_metadata,
  get_sentence_list_navigation,
} from "../../../yap-frontend-rs/pkg";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import { Card } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { Movies } from "@/browse/Movies";
import { languageToIso6391 } from "@/lib/utils";
import { acknowledgePimsleur, pimsleurAcknowledged } from "@/browse/pimsleur";

/**
 * Every curriculum, one tab per category. Picking an item is the choice:
 * it appends the switch and closes, so there is nothing left to confirm.
 * `category` is the open tab, or null when closed.
 */
export function CurriculumBrowser({
  deck,
  category,
  onClose,
  onChoose,
}: {
  deck: Deck;
  category: SentenceListCategory | null;
  onClose: () => void;
  onChoose: (event: DeckEvent) => void;
}) {
  return (
    <Dialog open={category !== null} onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="sm:max-w-3xl max-h-[85vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Choose what to study</DialogTitle>
        </DialogHeader>
        {category && (
          <BrowserTabs
            deck={deck}
            initial={category}
            onChoose={(selection) => {
              onChoose(deck.change_sentence_list(selection));
              onClose();
            }}
          />
        )}
      </DialogContent>
    </Dialog>
  );
}

function BrowserTabs({
  deck,
  initial,
  onChoose,
}: {
  deck: Deck;
  initial: SentenceListCategory;
  onChoose: (selection: SentenceListSelection | undefined) => void;
}) {
  const [tab, setTab] = useState(initial);
  const [acknowledged, setAcknowledged] = useState(pimsleurAcknowledged);
  // Computed once per opening: the movie stats are the costly part.
  const [lists] = useState(() => {
    const movies = deck.get_movie_stats();
    const metadata = new Map(
      deck.get_movie_metadata(movies.map((movie) => movie.id)).map((movie) => [movie.id, movie]),
    );
    const lessons = deck.get_pimsleur_stats();
    return {
      tier: deck.get_current_tier(),
      movies: movies.flatMap((stat) => {
        const movie = metadata.get(stat.id);
        return movie ? [{ ...movie, ...stat }] : [];
      }),
      lessons,
      navigation: get_sentence_list_navigation(
        deck.get_sentence_list(),
        movies.length > 0,
        lessons.length > 0,
      ),
    };
  });
  const selected = deck.get_sentence_list();
  const language = deck.get_target_language();
  const levels = [...new Set(lists.lessons.map((lesson) => lesson.level))].sort(
    (a, b) => a - b,
  );
  const labels: Record<SentenceListCategory, string> = {
    essential: "Essential",
    movie: "Movies",
    pimsleur: "Pimsleur",
  };
  const essentialTitle = `${lists.tier.name} ${get_language_metadata(language).commonName} Level ${lists.tier.level}`;

  return (
    <Tabs
      value={tab}
      onValueChange={(value) => setTab(value as SentenceListCategory)}
      className="gap-4"
    >
      <TabsList className="w-full">
        {lists.navigation.categories.map((category) => (
          <TabsTrigger
            key={category}
            value={category}
            className="data-[state=active]:bg-primary data-[state=active]:text-primary-foreground dark:data-[state=active]:bg-primary dark:data-[state=active]:text-primary-foreground"
          >
            {labels[category]}
          </TabsTrigger>
        ))}
      </TabsList>
      <TabsContent value="essential">
        <ListCard
          selected={selected === undefined}
          onClick={() => onChoose(undefined)}
          title={essentialTitle}
          percentKnown={lists.tier.percent_known}
          done={false}
        />
      </TabsContent>
      <TabsContent value="movie">
        <Movies
          moviesWithMetadata={lists.movies}
          targetLanguageIso={languageToIso6391(language)}
          deck={deck}
          selectedMovieId={selected?.type === "Movie" ? selected.id : undefined}
          onSelectMovie={(id) => onChoose({ type: "Movie", id })}
        />
      </TabsContent>
      <TabsContent value="pimsleur" className="space-y-4">
        {!acknowledged ? (
          <div className="flex flex-col items-center gap-3 py-4 text-center">
            <p className="text-sm text-muted-foreground">
              Yap has word lists for Pimsleur, but is not affiliated with
              Pimsleur in any way.
            </p>
            <Button
              onClick={() => {
                acknowledgePimsleur();
                setAcknowledged(true);
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
            {levels.map((level) => (
              <Collapsible
                key={level}
                defaultOpen={selected?.type === "PimsleurLesson" && selected.level === level}
                className="flex flex-col gap-2"
              >
                <CollapsibleTrigger className="flex items-center gap-2 w-full group">
                  <h4 className="text-sm font-semibold">Level {level}</h4>
                  <ChevronDown className="h-3 w-3 text-muted-foreground transition-transform group-data-[state=open]:rotate-180" />
                </CollapsibleTrigger>
                <CollapsibleContent className="space-y-2">
                  {lists.lessons
                    .filter((lesson) => lesson.level === level)
                    .map((lesson) => (
                      <ListCard
                        key={lesson.lesson}
                        selected={
                          selected?.type === "PimsleurLesson" &&
                          selected.level === lesson.level &&
                          selected.lesson === lesson.lesson
                        }
                        onClick={() =>
                          onChoose({
                            type: "PimsleurLesson",
                            level: lesson.level,
                            lesson: lesson.lesson,
                          })
                        }
                        title={`Lesson ${lesson.lesson}`}
                        percentKnown={lesson.percent_known}
                        done={lesson.all_available_learned}
                        icon
                      />
                    ))}
                </CollapsibleContent>
              </Collapsible>
            ))}
          </>
        )}
      </TabsContent>
    </Tabs>
  );
}

function ListCard({
  selected,
  onClick,
  title,
  percentKnown,
  done,
  icon = false,
}: {
  selected: boolean;
  onClick: () => void;
  title: string;
  percentKnown: number;
  done: boolean;
  icon?: boolean;
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
        {icon && <Headphones className="h-5 w-5 shrink-0 text-muted-foreground" />}
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
