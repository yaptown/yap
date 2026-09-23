import { useRef, useState, useEffect, useMemo } from "react";
import { getPosterDataUrl } from "@/lib/poster-utils";
import type { Deck } from "../../../yap-frontend-rs/pkg";

interface PosterProps {
  movieId: string;
  deck: Deck;
  alt: string | undefined;
}

export function Poster({ movieId, deck, alt }: PosterProps) {
  const ref = useRef<HTMLDivElement>(null);
  const [visible, setVisible] = useState(false);
  const bytes = visible ? deck.get_movie_poster(movieId) : undefined;
  // eslint-disable-next-line react-hooks/preserve-manual-memoization -- Bridgerton stable returns preserve byte identity.
  const posterDataUrl = useMemo(() => getPosterDataUrl(bytes), [bytes]);

  useEffect(() => {
    if (visible) return;

    const el = ref.current;
    if (!el) return;

    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          setVisible(true);
          observer.disconnect();
        }
      },
      { rootMargin: "200px" },
    );
    observer.observe(el);
    return () => observer.disconnect();
  }, [visible]);

  if (!visible) {
    return <div ref={ref} className="w-full h-full" />;
  }

  if (!posterDataUrl) {
    return (
      <div className="w-full h-full flex items-center justify-center text-4xl">
        🎬
      </div>
    );
  }

  return (
    <img src={posterDataUrl} alt={alt} className="w-full h-full object-cover" />
  );
}
