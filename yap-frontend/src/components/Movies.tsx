import { useState, useMemo } from 'react'
import type { Deck } from '../../../yap-frontend-rs/pkg'
import { Card } from "@/components/ui/card"
import { getMovieMetadata } from '@/lib/movie-cache'

interface MoviesProps {
  deck: Deck
}

export function Movies({ deck }: MoviesProps) {
  const movieStats = useMemo(() => deck.get_movie_stats(), [deck])
  const [showAllMovies, setShowAllMovies] = useState(false)

  // Join stats with metadata (metadata is cached globally)
  const moviesWithMetadata = useMemo(() => {
    const movieIds = movieStats.map(s => s.id)
    const metadata = getMovieMetadata(deck, movieIds)
    const metadataMap = new Map(metadata.map(m => [m.id, m]))

    return movieStats.map(stat => ({
      ...stat,
      ...(metadataMap.get(stat.id) || {}),
    }))
  }, [movieStats, deck])

  // Find movie closest to next milestone
  const closestToMilestone = useMemo(() => {
    return moviesWithMetadata
      .filter(m => m.cards_to_next_milestone !== null && m.cards_to_next_milestone !== undefined)
      .sort((a, b) => (a.cards_to_next_milestone || 0) - (b.cards_to_next_milestone || 0))[0]
  }, [moviesWithMetadata])

  const visibleMovies = showAllMovies ? moviesWithMetadata : moviesWithMetadata.slice(0, 8)

  // Helper function to convert poster bytes to data URL
  const getPosterDataUrl = (posterBytes: number[] | undefined) => {
    if (!posterBytes) return null
    const uint8Array = new Uint8Array(posterBytes)
    let binaryString = ''
    const chunkSize = 8192
    for (let i = 0; i < uint8Array.length; i += chunkSize) {
      const chunk = uint8Array.subarray(i, i + chunkSize)
      binaryString += String.fromCharCode(...chunk)
    }
    return `data:image/jpeg;base64,${btoa(binaryString)}`
  }

  if (movieStats.length === 0) {
    return null
  }

  return (
    <div className="mt-6">
      <h2 className="text-2xl font-semibold mb-3">Movies</h2>
      <p className="text-sm text-muted-foreground mb-4">
        These movies are sorted by how much of the dialogue you already know. You can usually watch a movie comfortably once you know 95% of the words.
      </p>

      {/* Featured movie closest to milestone */}
      {closestToMilestone && (
        <Card className="mb-6 overflow-hidden p-0 border-primary/50" animate>
          <div className="flex flex-col sm:flex-row gap-0">
            <div className="sm:w-32 w-full aspect-[2/3] sm:aspect-[2/3] bg-muted relative">
              {getPosterDataUrl(closestToMilestone.poster_bytes) ? (
                <img
                  src={getPosterDataUrl(closestToMilestone.poster_bytes)!}
                  alt={closestToMilestone.title}
                  className="w-full h-full object-cover"
                />
              ) : (
                <div className="w-full h-full flex items-center justify-center text-4xl">
                  🎬
                </div>
              )}
            </div>
            <div className="flex-1 p-4 flex flex-col justify-center">
              <div className="text-xs font-medium text-primary mb-1">ALMOST THERE</div>
              <h3 className="text-lg font-semibold mb-1">{closestToMilestone.title}</h3>
              {closestToMilestone.year && (
                <div className="text-sm text-muted-foreground mb-2">{closestToMilestone.year}</div>
              )}
              <p className="text-sm mb-3">
                You're just <span className="font-semibold text-foreground">{closestToMilestone.cards_to_next_milestone} {closestToMilestone.cards_to_next_milestone === 1 ? 'card' : 'cards'}</span> away from reaching <span className="font-semibold text-foreground">{Math.ceil(closestToMilestone.percent_known / 5) * 5}%</span> comprehension!
              </p>
              <div className="flex items-center gap-2">
                <div className="flex-1 h-2 bg-muted rounded-full overflow-hidden">
                  <div
                    className="h-full bg-primary transition-all duration-300"
                    style={{ width: `${closestToMilestone.percent_known}%` }}
                  />
                </div>
                <span className="text-xs font-mono font-semibold">
                  {closestToMilestone.percent_known.toFixed(0)}%
                </span>
              </div>
            </div>
          </div>
        </Card>
      )}

      <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-4">
        {visibleMovies.map((movie) => {
          const posterDataUrl = getPosterDataUrl(movie.poster_bytes)

          return (
            <Card
              key={movie.id}
              className="overflow-hidden p-0 hover:ring-2 hover:ring-primary transition-all cursor-pointer group gap-0"
              animate
            >
              <div className="relative aspect-[2/3] bg-muted">
                {posterDataUrl ? (
                  <img
                    src={posterDataUrl}
                    alt={movie.title}
                    className="w-full h-full object-cover"
                  />
                ) : (
                  <div className="w-full h-full flex items-center justify-center text-4xl">
                    🎬
                  </div>
                )}
                <div className="absolute inset-0 bg-gradient-to-t from-black/80 via-black/20 to-transparent opacity-0 group-hover:opacity-100 transition-opacity">
                  <div className="absolute bottom-0 left-0 right-0 p-3">
                    <div className="text-white text-sm font-semibold line-clamp-2">
                      {movie.title}
                    </div>
                    {movie.year && (
                      <div className="text-white/70 text-xs mt-1">
                        {movie.year}
                      </div>
                    )}
                    {movie.cards_to_next_milestone !== null && movie.cards_to_next_milestone !== undefined && (
                      <div className="text-white/90 text-xs mt-2 font-medium">
                        {movie.cards_to_next_milestone} {movie.cards_to_next_milestone === 1 ? 'card' : 'cards'} to {Math.ceil(movie.percent_known / 5) * 5}%
                      </div>
                    )}
                  </div>
                </div>
              </div>
              <div className="p-2 text-center relative overflow-hidden">
                <div
                  className="absolute inset-0 bg-foreground/10"
                  style={{
                    clipPath: `inset(0 ${100 - movie.percent_known}% 0 0)`
                  }}
                />
                <span className="relative text-sm font-mono font-semibold text-foreground">
                  {movie.percent_known.toFixed(0)}% known
                </span>
              </div>
            </Card>
          );
        })}
      </div>
      {!showAllMovies && movieStats.length > 10 && (
        <div className="mt-4">
          <button
            onClick={() => setShowAllMovies(true)}
            className="w-full py-3 text-sm text-muted-foreground hover:text-foreground hover:bg-muted/50 transition-colors duration-200 font-medium rounded-md border border-border"
          >
            Show all {movieStats.length} movies
          </button>
        </div>
      )}
    </div>
  )
}
