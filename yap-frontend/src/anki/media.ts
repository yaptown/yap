import type { AnkiDeckPlan, AnkiMediaSource } from "../../../yap-frontend-rs/pkg";

// The backend paces itself against Gemini's and Whisper's per-minute quotas,
// so the fan-out here only needs to be wide enough to keep those saturated;
// it is not what protects the providers. The failure cap still stops a
// broken export before it floods the service.
const CONCURRENCY = 24;
const TIMEOUT_MS = 60_000;
const RETRIES = 1;
const MAX_FAILURES = 32;

export type MediaProgress = { done: number; total: number };

export class AudioUnavailableError extends Error {
  readonly code = "audio_unavailable";
  constructor() { super("audio_unavailable"); }
}

/** Bound both headers and body, and let a fatal failure cancel all workers. */
async function recording(url: string, controller: AbortController): Promise<Uint8Array | undefined> {
  const { signal } = controller;
  for (let attempt = 0; attempt <= RETRIES; attempt++) {
    signal.throwIfAborted();
    const request = new AbortController();
    const abort = () => request.abort(signal.reason);
    signal.addEventListener("abort", abort, { once: true });
    const timer = setTimeout(() => request.abort(new Error("Audio request timed out")), TIMEOUT_MS);
    try {
      const response = await fetch(url, { signal: request.signal });
      if (response.status >= 400 && response.status < 500 && response.status !== 429) return undefined;
      if (!response.ok) throw new Error(`Audio request failed (${response.status})`);
      const bytes = new Uint8Array(await response.arrayBuffer());
      if (!bytes.length) throw new Error("Audio response was empty");
      return bytes;
    } catch {
      signal.throwIfAborted();
      if (attempt === RETRIES) return undefined;
    } finally {
      request.abort();
      clearTimeout(timer);
      signal.removeEventListener("abort", abort);
    }
  }
}

/** Results stay in plan order, regardless of request completion order. */
export async function fetchMedia(
  plan: AnkiDeckPlan,
  fetchBundled: (source: AnkiMediaSource) => Uint8Array | undefined | Promise<Uint8Array | undefined>,
  onProgress: (progress: MediaProgress) => void,
): Promise<(Uint8Array | undefined)[]> {
  const controller = new AbortController();
  const results = new Array<Uint8Array | undefined>(plan.bundled.length);
  let next = 0;
  let done = 0;
  let failures = 0;
  onProgress({ done, total: results.length });
  try {
    await Promise.all(Array.from({ length: Math.min(CONCURRENCY, results.length) }, async () => {
      while (next < results.length) {
        controller.signal.throwIfAborted();
        const index = next++;
        const { source, filename } = plan.bundled[index];
        const bytes = source.type === "Tts"
          ? await recording(source.url, controller)
          : await fetchBundled(source);
        controller.signal.throwIfAborted();
        if (!bytes && source.type !== "Subtitles") {
          if (source.type !== "Tts") throw new Error(`Bundled media missing: ${filename}`);
          if (++failures >= MAX_FAILURES) {
            controller.abort(new AudioUnavailableError());
            controller.signal.throwIfAborted();
          }
        }
        results[index] = bytes;
        onProgress({ done: ++done, total: results.length });
      }
    }));
  } catch (error) {
    controller.abort(error);
    throw error;
  }
  return results;
}
