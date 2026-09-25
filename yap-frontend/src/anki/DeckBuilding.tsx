import { useEffect, useState } from "react";
import { AnimatePresence, motion, useReducedMotion } from "framer-motion";
import { Film } from "lucide-react";
import type { Deck, Language } from "../../../yap-frontend-rs/pkg";
import { Poster } from "@/browse/Poster";
import { TargetLanguageText } from "@/components/TargetLanguageText";
import { Progress } from "@/components/ui/progress";
import type { DeckBuild } from "./deck-build";

const FEED = 4;
const SHELF = 9;

/**
 * The deck assembling itself: sentences drop in beside a growing card stack,
 * and each new film's poster joins the shelf. Once every sentence is chosen
 * (`choosing` false) the feed slowly replays the deck while media downloads.
 */
export function DeckBuilding({ build, deck, language, choosing, animate, status, progress, target, finishMessage }: {
  build: DeckBuild;
  deck: Deck;
  language: Language;
  choosing: boolean;
  /** False once the deck is downloaded: everything holds still. */
  animate: boolean;
  status?: string;
  progress?: number;
  /** Sentences the planner is aiming for; only while choosing. */
  target?: number;
  finishMessage?: string;
}) {
  const reduceMotion = useReducedMotion();
  const { sentences, films } = build;
  // The planner picks sentences in bursts, far faster than anyone can read,
  // so the feed reveals them one at a time at its own pace. Once every
  // sentence is chosen it keeps going round the deck until the download ends.
  const [cursor, setCursor] = useState(0);
  const looping = !choosing && sentences.length > FEED;
  const ticking = animate && !reduceMotion && (cursor < sentences.length || looping);
  useEffect(() => {
    if (!ticking) return;
    const timer = setInterval(() => setCursor((value) => value + 1), 450);
    return () => clearInterval(timer);
  }, [ticking]);
  const newest = animate && !reduceMotion ? cursor : sentences.length;
  const feed = newest <= sentences.length
    ? sentences.slice(Math.max(0, newest - FEED), newest).reverse()
    : Array.from({ length: FEED }, (_, k) => sentences[(newest - 1 - k) % sentences.length]);
  const shelf = films.filter((film) => film.poster).slice(-SHELF);

  return (
    <div className="flex flex-col gap-5 rounded-xl border bg-card/60 p-5 backdrop-blur-sm">
      <div className="flex items-stretch gap-4">
        <div role="list" className="relative flex h-52 min-w-0 flex-1 flex-col gap-2 overflow-hidden [mask-image:linear-gradient(to_bottom,black_60%,transparent)]" aria-label="Sentences in your deck">
          <AnimatePresence initial={false} mode="popLayout">
            {feed.map((note, index) => (
              <motion.div
                role="listitem"
                key={note.guid}
                layout={!reduceMotion}
                initial={{ opacity: 0, y: -18, scale: 0.97 }}
                animate={{ opacity: 1 - index * 0.18, y: 0, scale: 1 }}
                exit={{ opacity: 0, y: 12 }}
                transition={{ type: "spring", stiffness: 380, damping: 32 }}
                className="flex items-center gap-3 rounded-lg border border-border/60 bg-background/70 p-2 shadow-sm"
              >
                <div className="flex h-11 w-8 shrink-0 items-center justify-center overflow-hidden rounded-sm bg-muted">
                  {note.source.poster_filename ? <Poster movieId={note.source.imdb_id} deck={deck} alt="" /> : <Film className="size-4 text-muted-foreground" aria-hidden />}
                </div>
                <div className="flex min-w-0 flex-col">
                  <span className="truncate text-sm font-medium">
                    <TargetLanguageText language={language}>{note.sentence}</TargetLanguageText>
                  </span>
                  <span className="truncate text-xs text-muted-foreground">{note.source.title}</span>
                </div>
              </motion.div>
            ))}
          </AnimatePresence>
        </div>
        <DeckStack count={sentences.length} />
      </div>

      <div className="flex flex-col gap-3">
        <div className="flex h-12 -space-x-3">
          <AnimatePresence initial={false}>
            {shelf.map((film) => (
              <motion.div
                key={film.id}
                title={film.title}
                initial={{ opacity: 0, scale: 0.6, y: 6 }}
                animate={{ opacity: 1, scale: 1, y: 0 }}
                transition={{ type: "spring", stiffness: 420, damping: 22 }}
                className="h-12 w-8 overflow-hidden rounded-sm border-2 border-card bg-muted shadow-sm"
              >
                <Poster movieId={film.id} deck={deck} alt={film.title} />
              </motion.div>
            ))}
          </AnimatePresence>
        </div>
        <p className="font-mono text-xs text-muted-foreground tabular-nums">
          {sentences.length.toLocaleString()}{target ? `/${target.toLocaleString()}` : ""} sentences{build.words ? ` · ${build.words.toLocaleString()} words` : ""} · {films.length.toLocaleString()} films
        </p>
      </div>

      {status && (
        <div className="flex flex-col gap-2 text-sm text-muted-foreground">
          <p>{status}</p>
          {progress !== undefined && <Progress value={progress} />}
        </div>
      )}
      {finishMessage && <p className="animate-fade-in text-lg font-semibold text-pretty">{finishMessage}</p>}
    </div>
  );
}

// Home's card stack, holding the sentences chosen so far.
function DeckStack({ count }: { count: number }) {
  return (
    <div className="relative w-24 shrink-0 self-center sm:w-28" style={{ height: "8.5rem" }} aria-hidden>
      <div className="absolute inset-0 translate-x-3 rotate-[10deg] rounded-2xl border border-border/60 bg-card/30 backdrop-blur-sm" />
      <div className="absolute inset-0 translate-x-1 rotate-[3deg] rounded-2xl border border-border/60 bg-card/45 backdrop-blur-sm" />
      <div className="absolute inset-0 -rotate-6 flex flex-col items-center justify-center rounded-2xl border border-border/60 bg-card/70 shadow-sm backdrop-blur-md">
        <motion.span key={count} initial={{ scale: 1.25 }} animate={{ scale: 1 }} className="text-3xl font-bold tabular-nums">
          {count}
        </motion.span>
        <span className="text-xs text-muted-foreground">sentences</span>
      </div>
    </div>
  );
}
