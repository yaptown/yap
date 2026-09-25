import type { AnkiDeckPlan, AnkiMediaSource } from "../../../yap-frontend-rs/pkg";
import { buildApkg, type MediaProgress } from "./apkg";

export type PackageRequest =
  | { type: "build"; plan: AnkiDeckPlan; modified: number }
  | { type: "media"; id: number; bytes: Uint8Array | undefined };
export type PackageResponse =
  | { type: "media"; id: number; source: AnkiMediaSource }
  | { type: "progress"; progress: MediaProgress }
  | { type: "complete"; blob: Blob }
  | { type: "error"; message: string };

const pending = new Map<number, (bytes: Uint8Array | undefined) => void>();
let nextId = 0;
const send = (message: PackageResponse) => self.postMessage(message);

self.onmessage = async ({ data }: MessageEvent<PackageRequest>) => {
  if (data.type === "media") {
    pending.get(data.id)!(data.bytes);
    pending.delete(data.id);
    return;
  }
  try {
    const blob = await buildApkg(data.plan, (source) => new Promise((resolve) => {
      const id = nextId++;
      pending.set(id, resolve);
      send({ type: "media", id, source });
    }), (progress) => send({ type: "progress", progress }), data.modified);
    // Blob cloning shares its immutable backing storage, not a main-thread copy.
    send({ type: "complete", blob });
  } catch (error) {
    send({ type: "error", message: error instanceof Error ? error.message : String(error) });
  }
};
