import {
  get_audio,
  get_temp_audio,
  invalidate_audio_cache,
  type AudioRequest,
  type VoiceActorInfo,
} from "../../../yap-frontend-rs/pkg";

import type { PlaybackOptions } from "./pure";

export type { VoiceActorInfo };
export * from "./pure";

// One media element plays at a time, app-wide. TTS playback and the movie
// clip player both register their interrupt here; starting either one stops
// whatever else was playing.
let interruptCurrent: (() => void) | undefined;

/// Stop whatever is currently playing (TTS audio or a movie clip).
export function interruptPlayback(): void {
  interruptCurrent?.();
}

/// Register the interrupt for a playback that is about to start. Returns an
/// unregister function; call it when the playback ends on its own so a stale
/// interrupt can't fire later.
export function registerPlayback(interrupt: () => void): () => void {
  interruptCurrent = interrupt;
  return () => {
    if (interruptCurrent === interrupt) interruptCurrent = undefined;
  };
}

export async function playAudio(
  audioRequest: AudioRequest,
  accessToken: string | undefined,
  needsAuth: () => void,
  { temporary = false, onAudioElement, signal, onVoiceActor }: PlaybackOptions = {},
): Promise<void> {
  const abortError = () => new DOMException("Aborted", "AbortError");
  if (signal?.aborted) throw abortError();
  interruptPlayback();

  return new Promise((resolve, reject) => {
    let audio: HTMLAudioElement | undefined;
    let audioUrl: string | undefined;
    let settled = false;

    const finish = (error?: unknown) => {
      if (settled) return;
      settled = true;
      unregister();
      signal?.removeEventListener("abort", interrupt);
      if (audio) {
        // Clearing src can fire an error; detach handlers before teardown.
        audio.onended = null;
        audio.onerror = null;
        try {
          audio.pause();
          audio.src = "";
        } catch {
          // The element may already be detached.
        }
      }
      if (audioUrl) URL.revokeObjectURL(audioUrl);
      if (error === undefined) resolve();
      else reject(error);
    };

    const interrupt = () => finish(abortError());
    const unregister = registerPlayback(interrupt);
    signal?.addEventListener("abort", interrupt, { once: true });

    const playbackFailed = (error: unknown) => {
      if (settled) return;
      const failure = error instanceof Error ? error : new Error(String(error));
      finish(failure);
      if (!temporary && failure.name !== "NotAllowedError") {
        void invalidate_audio_cache(audioRequest).catch((error) => {
          console.error("Failed to invalidate audio cache:", error);
        });
      }
    };

    const start = async () => {
      const result = await (temporary ? get_temp_audio : get_audio)(
        audioRequest,
        accessToken,
      );
      if (settled) return;
      if (result.voice_actor) onVoiceActor?.(result.voice_actor);
      if (settled) return;

      audioUrl = URL.createObjectURL(
        new Blob([result.bytes], { type: "audio/mpeg" }),
      );
      audio = new Audio(audioUrl);
      if (onAudioElement) await onAudioElement(audio);
      if (settled) return;

      audio.onended = () => finish();
      audio.onerror = () => playbackFailed(new Error("Audio playback failed"));
      try {
        await audio.play();
      } catch (error) {
        playbackFailed(error);
      }
    };

    void start().catch((error) => {
      if (settled) return;
      finish(error ?? new Error("Audio preparation failed"));
      if (typeof error === "string" && error.includes("400")) needsAuth();
      console.error("Failed to prepare audio:", error);
    });
  });
}
