import { t } from "@/lib/i18n";
import { useState, useDeferredValue } from "react";
import { MoviePosterCard } from "@/review/challenges/MoviePosterCard";
import type { Deck, MovieMetadataBasic } from "../../../yap-frontend-rs/pkg";

interface MovieWithMetadata extends MovieMetadataBasic {
  percent_known: number;
  cards_to_next_milestone: number | null | undefined;
}

interface MoviesProps {
  moviesWithMetadata: MovieWithMetadata[];
  targetLanguageIso: string;
  deck: Deck;
  selectedMovieId?: string;
  onSelectMovie: (id: string) => void;
}

export function Movies({
  moviesWithMetadata: moviesWithMetadataProp,
  targetLanguageIso,
  deck,
  selectedMovieId,
  onSelectMovie,
}: MoviesProps) {
  const moviesWithMetadata = useDeferredValue(moviesWithMetadataProp);
  const [showAllMovies, setShowAllMovies] = useState(false);

  // The current movie first, then films made in the language.
  const rank = (movie: MovieWithMetadata) =>
    movie.id === selectedMovieId
      ? 0
      : movie.original_language === targetLanguageIso
        ? 1
        : 2;
  const sortedMovies = [...moviesWithMetadata].sort(
    (a, b) => rank(a) - rank(b),
  );
  const visibleMovies = showAllMovies
    ? sortedMovies
    : sortedMovies.slice(0, 12);

  if (moviesWithMetadata.length === 0) {
    return null;
  }

  return (
    <div className="flex flex-col gap-4">
      <p className="text-sm text-muted-foreground">
        {t("movies-you-can-usually-watch-a-movie-comfortably-once-you")}
      </p>
      <div className="grid grid-cols-3 sm:grid-cols-4 gap-3">
        {visibleMovies.map((movie) => (
          <MoviePosterCard
            key={movie.id}
            movie={movie}
            deck={deck}
            className={`relative ${selectedMovieId === movie.id ? "ring-2 ring-primary" : ""}`}
            overlayExtra={
              movie.cards_to_next_milestone !== null &&
              movie.cards_to_next_milestone !== undefined && (
                <div className="text-white/90 text-xs mt-2 font-medium">
                  {t("movies-next-milestone", {
                    count: movie.cards_to_next_milestone,
                    percent: Math.ceil(movie.percent_known / 5) * 5,
                  })}
                </div>
              )
            }
          >
            <button
              type="button"
              className="absolute inset-0 z-10 rounded-xl focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-inset"
              aria-label={movie.title}
              aria-pressed={selectedMovieId === movie.id}
              onClick={() => onSelectMovie(movie.id)}
            />
            <div className="p-1.5 text-center relative overflow-hidden">
              <div
                className="absolute inset-0 bg-foreground/10"
                style={{
                  clipPath: `inset(0 ${100 - movie.percent_known}% 0 0)`,
                }}
              />
              <span className="relative text-xs font-mono font-semibold text-foreground">
                {t("curriculum-percent-known", {
                  percent: Math.floor(movie.percent_known),
                })}
              </span>
            </div>
          </MoviePosterCard>
        ))}
      </div>
      {!showAllMovies && sortedMovies.length > visibleMovies.length && (
        <div>
          <button
            onClick={() => setShowAllMovies(true)}
            className="w-full py-3 text-sm text-muted-foreground hover:text-foreground hover:bg-muted/50 transition-colors duration-200 font-medium rounded-md border border-border"
          >
            {t("curriculum-show-all-movies", {
              count: sortedMovies.length,
            })}
          </button>
        </div>
      )}
    </div>
  );
}
