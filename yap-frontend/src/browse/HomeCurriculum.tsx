import { useState } from "react";
import { Check, ChevronDown, Headphones } from "lucide-react";
import type {
  Deck,
  DeckEvent,
  HomeCurriculumView,
  SentenceListCategory,
} from "../../../yap-frontend-rs/pkg";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { Poster } from "@/browse/Poster";
import { TargetLanguageText } from "@/components/TargetLanguageText";
import { CurriculumBrowser } from "@/browse/CurriculumBrowser";
import { pimsleurAcknowledged } from "@/browse/pimsleur";

/**
 * What Home is studying from. The name is a menu of categories; a movie or
 * lesson opens the browser for the rest of its category. Both commit on the spot.
 */
export function HomeCurriculum({
  view,
  deck,
  inert,
  onChoose,
}: {
  view: HomeCurriculumView;
  deck: Deck;
  inert: boolean;
  onChoose: (event: DeckEvent) => void;
}) {
  const [browsing, setBrowsing] = useState<SentenceListCategory | null>(null);
  const choose = (category: SentenceListCategory) => {
    if (inert || category === view.category) return;
    // The disclaimer lives in the browser, so an unacknowledged Pimsleur goes there first.
    if (category === "pimsleur" && !pimsleurAcknowledged()) setBrowsing(category);
    else onChoose(deck.choose_curriculum_category(category));
  };
  const item = view.item;
  return (
    <section className="flex flex-col gap-3 p-5">
      <div className="flex items-center gap-3">
        <DropdownMenu>
          <DropdownMenuTrigger className="-mx-1.5 flex items-center gap-1 rounded-md px-1.5 text-lg font-semibold hover:bg-muted/60 transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring data-[state=open]:bg-muted/60">
            {view.menu_label}
            <ChevronDown className="h-4 w-4 text-muted-foreground" aria-hidden />
          </DropdownMenuTrigger>
          <DropdownMenuContent
            align="start"
            className="min-w-48"
            onCloseAutoFocus={(event) => event.preventDefault()}
          >
            {view.options.map((option) => (
              <DropdownMenuItem
                key={option.category}
                onSelect={() => choose(option.category)}
                className="py-2 text-base"
              >
                {option.label}
                {option.category === view.category && (
                  <Check className="ml-auto h-4 w-4 text-foreground" />
                )}
              </DropdownMenuItem>
            ))}
          </DropdownMenuContent>
        </DropdownMenu>
        {view.level_label && (
          <span className="rounded-full border border-border px-2.5 py-0.5 text-xs text-muted-foreground whitespace-nowrap">
            {view.level_label}
          </span>
        )}
      </div>
      {item ? (
        <div className="flex items-center gap-4">
          <div className="h-[72px] w-12 shrink-0 overflow-hidden rounded-md border border-border/50 bg-muted flex items-center justify-center">
            {item.movie_id ? (
              <Poster movieId={item.movie_id} deck={deck} alt={item.title} />
            ) : (
              <Headphones className="h-5 w-5 text-muted-foreground" />
            )}
          </div>
          <div className="flex min-w-0 flex-1 flex-col gap-2">
            <div className="flex items-center gap-2">
              <p className="min-w-0 flex-1 truncate font-medium">
                {item.movie_id ? (
                  <TargetLanguageText language={deck.get_target_language()}>
                    {item.title}
                  </TargetLanguageText>
                ) : (
                  item.title
                )}
              </p>
              <Button
                variant="outline"
                size="sm"
                className="h-7 shrink-0 px-2.5"
                onClick={() => !inert && setBrowsing(view.category)}
              >
                {item.browse_label}
              </Button>
            </div>
            <ProgressRow view={view} label={item.title} />
          </div>
        </div>
      ) : (
        <ProgressRow view={view} label={view.menu_label} />
      )}
      {view.subtitle && (
        <p className="text-sm text-muted-foreground">{view.subtitle}</p>
      )}
      <CurriculumBrowser
        deck={deck}
        category={browsing}
        onClose={() => setBrowsing(null)}
        onChoose={onChoose}
      />
    </section>
  );
}

function ProgressRow({ view, label }: { view: HomeCurriculumView; label: string }) {
  return (
    <Progress
      value={view.percent}
      label={view.percent_label}
      aria-label={label}
      className="h-6"
    />
  );
}
