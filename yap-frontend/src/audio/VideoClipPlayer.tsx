import {
  useState,
  useEffect,
  useRef,
  useCallback,
  type ReactNode,
} from "react";
import { Play } from "lucide-react";
import {
  get_clip,
  get_clip_manifest_version,
  invalidate_clip_cache,
  type ClipSubtitleCue,
  type Deck,
  type Language,
} from "../../../yap-frontend-rs/pkg";
import { interruptPlayback, registerPlayback } from "@/lib/utils";
import { isSoundEffectPlaying } from "@/lib/sound-effects";
import { getMovieMetadata } from "@/lib/movie-cache";
import { TargetLanguageText } from "../components/TargetLanguageText";
import { Poster } from "../browse/Poster";

interface VideoClipPlayerProps {
  language: Language;
  text: string;
  accessToken: string | undefined;
  /** Autoplay once when this becomes true (shared-flag protocol below). */
  autoPlay?: boolean;
  /** Shared per-challenge "already autoplayed" flag, same as AudioButton's. */
  autoplayed?: boolean;
  setAutoplayed?: () => void;
  /**
   * Reports the clip’s film (or null), so the parent can avoid duplicate posters and
   * decide who owns autoplay (video when available, AudioButton otherwise).
   */
  onClipChange?: (movieId: string | null) => void;
  /**
   * Custom rendering for the target sentence's own caption cue (the
   * `"sentence"` role) — e.g. the transcription challenge masks the blanked
   * words so the caption doesn't give the answer away. Context cues always
   * render verbatim; they're neighboring lines, not the answer.
   */
  renderSentenceCue?: (text: string) => ReactNode;
  /**
   * When given, a subtle header over the top of the video names the film:
   * a small text-free poster, the title and year, and the Rotten Tomatoes
   * score. The film is identified by the clip itself, not the challenge — a
   * sentence can appear in several movies, but the clip was cut from
   * exactly one.
   */
  deck?: Deck;
}

type ClipState =
  | { status: "loading" }
  | { status: "unavailable" }
  | {
      status: "ready";
      url: string;
      criticalStartMs: number;
      subtitles: ClipSubtitleCue[];
      movieId: string;
    };

/**
 * The movie clip a sentence was cut from, as the card's primary media. The
 * clip arrives as bytes from the wasm cache (which mediates everything
 * through the AI backend) and plays from a blob URL. Renders nothing when
 * the sentence has no clip — the TTS AudioButton remains as the fallback.
 */
export function VideoClipPlayer({
  language,
  text,
  accessToken,
  autoPlay = false,
  autoplayed,
  setAutoplayed,
  onClipChange,
  renderSentenceCue,
  deck,
}: VideoClipPlayerProps) {
  const [clip, setClip] = useState<ClipState>({ status: "loading" });
  const [isPlaying, setIsPlaying] = useState(false);
  const [currentTimeMs, setCurrentTimeMs] = useState(0);
  const videoRef = useRef<HTMLVideoElement | null>(null);
  const unregisterRef = useRef<(() => void) | undefined>(undefined);
  const hasPlayedRef = useRef(false);

  const onClipChangeRef = useRef(onClipChange);
  useEffect(() => {
    onClipChangeRef.current = onClipChange;
  });

  // "No clip" can mean "no manifest yet": on a fresh session the challenge
  // may render before the background manifest refresh lands. Poll the
  // manifest version while we don't have a clip, so its arrival re-runs the
  // lookup; same-value updates bail, so the steady state costs nothing.
  const [manifestVersion, setManifestVersion] = useState(0);
  useEffect(() => {
    if (clip.status === "ready") return;
    const interval = setInterval(
      () => setManifestVersion(get_clip_manifest_version()),
      2000,
    );
    return () => clearInterval(interval);
  }, [clip.status]);

  useEffect(() => {
    let cancelled = false;
    let url: string | undefined;
    hasPlayedRef.current = false;

    void (async () => {
      try {
        const result = await get_clip(language, text, accessToken);
        if (cancelled) return;
        if (result === undefined || result === null) {
          setClip({ status: "unavailable" });
          onClipChangeRef.current?.(null);
          return;
        }
        // Copy into a fresh Uint8Array: the wasm-bindgen bytes are typed
        // over ArrayBufferLike, which newer DOM typings reject as a BlobPart.
        url = URL.createObjectURL(
          new Blob([new Uint8Array(result.bytes)], { type: "video/mp4" }),
        );
        setClip({
          status: "ready",
          url,
          criticalStartMs: Number(result.critical_start_ms),
          subtitles: result.subtitles,
          movieId: result.movie_id,
        });
        onClipChangeRef.current?.(result.movie_id);
      } catch (error) {
        console.error("Failed to fetch movie clip:", error);
        if (!cancelled) {
          setClip({ status: "unavailable" });
          onClipChangeRef.current?.(null);
        }
      }
    })();

    return () => {
      cancelled = true;
      if (url) URL.revokeObjectURL(url);
      unregisterRef.current?.();
      unregisterRef.current = undefined;
    };
    // Refetch when the sentence changes, or when a manifest arrives that
    // might now know a clip for it.
  }, [language, text, accessToken, manifestVersion]);

  const play = useCallback(() => {
    const video = videoRef.current;
    if (!video) return;
    // Stop whatever else is playing (TTS or another clip), then register
    // ourselves so a later playAudio can stop us in turn.
    interruptPlayback();
    unregisterRef.current = registerPlayback(() => video.pause());
    // Every play starts from the top — a short clip is a whole thought, and
    // resuming mid-sentence is never what a replay tap wants. (This also
    // clears the poster-frame seek into the critical window.)
    video.currentTime = 0;
    hasPlayedRef.current = true;
    video.play().catch((error) => {
      if (error instanceof Error && error.name === "NotAllowedError") return;
      console.error("Failed to play movie clip:", error);
    });
  }, []);

  // Autoplay mirrors AudioButton: once per challenge, only after the user
  // has interacted with the page, and never over a sound effect.
  useEffect(() => {
    if (!autoPlay || autoplayed || clip.status !== "ready") return;
    let cancelled = false;
    void (async () => {
      if (!navigator.userActivation?.hasBeenActive || !document.hasFocus()) {
        return;
      }
      while (isSoundEffectPlaying() && !cancelled) {
        await new Promise((resolve) => setTimeout(resolve, 50));
      }
      if (cancelled) return;
      setAutoplayed?.();
      play();
    })();
    return () => {
      cancelled = true;
    };
  }, [autoPlay, autoplayed, clip.status, setAutoplayed, play]);

  if (clip.status !== "ready") return null;

  // Time-synced caption: the cue overlapping the playhead, if any. Cues are
  // clip-relative ms straight from the sidecar; onTimeUpdate's ~4Hz cadence
  // is plenty for subtitle boundaries.
  const currentCue = clip.subtitles.find(
    (cue) => currentTimeMs >= Number(cue.at_ms) && currentTimeMs < Number(cue.until_ms),
  );

  const movie = deck ? getMovieMetadata(deck, [clip.movieId])[0] : undefined;

  return (
    <div className="relative animate-feedback-in">
      <div className="relative w-full overflow-hidden rounded-lg">
        <video
        ref={videoRef}
        src={clip.url}
        playsInline
        preload="auto"
        className="w-full cursor-pointer"
        onClick={() => {
          const video = videoRef.current;
          if (!video) return;
          if (video.paused) play();
          else video.pause();
        }}
        onLoadedMetadata={() => {
          const video = videoRef.current;
          if (video && !hasPlayedRef.current) {
            // Poster frame from inside the sentence, not black frame zero.
            video.currentTime = clip.criticalStartMs / 1000;
          }
        }}
        onTimeUpdate={() => {
          const video = videoRef.current;
          if (video) setCurrentTimeMs(video.currentTime * 1000);
        }}
        onPlay={() => setIsPlaying(true)}
        onPause={() => {
          setIsPlaying(false);
          unregisterRef.current?.();
          unregisterRef.current = undefined;
        }}
        onError={() => {
          // Cached bytes the element can't decode: forget them so the next
          // load refetches, and drop back to the audio-only layout.
          setClip({ status: "unavailable" });
          onClipChangeRef.current?.(null);
          void invalidate_clip_cache(language, text).catch((error) => {
            console.error("Failed to invalidate clip cache:", error);
          });
        }}
      />
      {isPlaying && currentCue && (
        <div className="pointer-events-none absolute inset-x-0 bottom-2 flex justify-center px-3">
          <span
            className={`rounded bg-black/60 px-2.5 py-1 text-center text-lg font-medium ${
              // The target sentence's own line stands apart from the
              // context dialogue around it.
              currentCue.role === "sentence" ? "text-amber-300" : "text-white/75"
            }`}
          >
            {currentCue.role === "sentence" && renderSentenceCue ? (
              renderSentenceCue(currentCue.text)
            ) : (
              <TargetLanguageText language={language}>
                {currentCue.text}
              </TargetLanguageText>
            )}
          </span>
        </div>
      )}
      {!isPlaying && (
        <button
          type="button"
          aria-label="Play clip"
          className="absolute inset-0 flex items-center justify-center bg-black/30"
          onClick={play}
        >
          <span className="flex h-14 w-14 items-center justify-center rounded-full bg-black/50 text-white">
            <Play className="h-7 w-7 translate-x-0.5" fill="currentColor" />
          </span>
        </button>
      )}
      </div>
      {movie && deck && (
        <div className="pointer-events-none absolute inset-x-0 top-0 rounded-t-lg bg-gradient-to-b from-black/60 via-black/30 to-transparent pb-4">
          {/* The row rides slightly above the frame: the poster pokes past
              the top edge and pulls the title up with it. */}
          <div className="-mt-6 flex items-center gap-2 px-2">
            <div className="pointer-events-auto -ml-4 w-15 shrink-0 aspect-[2/3] overflow-hidden rounded-md bg-muted shadow-md transition-transform origin-top-left rotate-4 hover:scale-[3] hover:rotate-0 hover:z-10">
              <Poster movieId={clip.movieId} deck={deck} alt={movie.title} />
            </div>
            <span className="min-w-0 truncate text-sm font-medium text-white [text-shadow:0_1px_3px_rgba(0,0,0,0.9)]">
              {movie.title}
              {movie.year !== undefined && (
                <span className="text-white/80"> ({movie.year})</span>
              )}
            </span>
            {movie.rotten_tomatoes_score !== undefined && (
              <span className="ml-auto shrink-0 pr-1 text-sm text-white [text-shadow:0_1px_3px_rgba(0,0,0,0.9)]">
                🍅 {movie.rotten_tomatoes_score}%
              </span>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
