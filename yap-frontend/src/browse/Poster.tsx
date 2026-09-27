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
  const posterDataUrl = usePosterDataUrl(visible ? deck.get_movie_poster(movieId) : undefined);

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

// Cache base64 conversion by the bridge's stable bytes, even across Deck snapshots.
function usePosterDataUrl(bytes: Uint8Array | undefined) {
  return useMemo(() => getPosterDataUrl(bytes), [bytes]);
}
