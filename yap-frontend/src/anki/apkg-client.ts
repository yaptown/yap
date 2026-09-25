import type { AnkiDeckPlan, AnkiMediaSource } from "../../../yap-frontend-rs/pkg";
import type { MediaProgress } from "./apkg";
import type { PackageRequest, PackageResponse } from "./apkg-worker";

// Anki compares whole seconds, even across separate workers/quick exports.
let lastModified = 0;
export function nextModificationTime(): number {
  lastModified = Math.max(Math.floor(Date.now() / 1000), lastModified + 1);
  return lastModified;
}

/** Only pack-backed media crosses the main thread; fetch, SQL, ZIP and Blob
 * creation all run in the worker. The deck itself never leaves this thread. */
export async function buildApkg(
  plan: AnkiDeckPlan,
  fetchBundled: (source: AnkiMediaSource) => Uint8Array | undefined,
  onProgress: (progress: MediaProgress) => void,
): Promise<Blob> {
  const worker = new Worker(new URL("./apkg-worker.ts", import.meta.url), { type: "module" });
  try {
    return await new Promise<Blob>((resolve, reject) => {
      const send = (message: PackageRequest, transfer: Transferable[] = []) => worker.postMessage(message, transfer);
      worker.onerror = (event) => reject(new Error(event.message));
      worker.onmessageerror = () => reject(new Error("Could not read the Anki worker response."));
      worker.onmessage = ({ data }: MessageEvent<PackageResponse>) => {
        try {
          switch (data.type) {
            case "media": {
              const bytes = fetchBundled(data.source);
              // Bridge-returned bytes are owned JS arrays, not WASM memory.
              send({ type: "media", id: data.id, bytes }, bytes ? [bytes.buffer as ArrayBuffer] : []);
              break;
            }
            case "progress": onProgress(data.progress); break;
            case "complete": resolve(data.blob); break;
            case "error": reject(new Error(data.message)); break;
          }
        } catch (error) {
          reject(error);
        }
      };
      send({ type: "build", plan, modified: nextModificationTime() });
    });
  } finally {
    worker.terminate();
  }
}
