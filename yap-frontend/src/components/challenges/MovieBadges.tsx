interface MovieBadgesProps {
  movieTitles: [string, string][];
}

export function MovieBadges({ movieTitles }: MovieBadgesProps) {
  if (!movieTitles || movieTitles.length === 0) {
    return null;
  }

  return (
    <div className="flex flex-wrap gap-1 justify-center mt-1">
      {movieTitles.map(([movieId, movieTitle]) => (
        <span
          key={movieId}
          className="text-xs px-2 py-0.5 rounded-full bg-primary/10 text-primary font-medium"
        >
          🎬 {movieTitle}
        </span>
      ))}
    </div>
  );
}
