import { useSyncExternalStore } from "react";
import type { MediaProgress } from "./apkg";
import { emptyBuild, type DeckBuild } from "./deck-build";

/** One course's deck export, from Generate to the finished file. */
export type AnkiExport = {
  /** Who started it; undefined for a signed-out visitor. */
  owner?: string;
  /** Set while building; undefined when idle or finished. */
  phase?: string;
  progress?: MediaProgress;
  choosing: boolean;
  build: DeckBuild;
  /** Bumped per Generate, so the animation starts fresh. */
  run: number;
  finishMessage?: string;
  /** "300 sentence notes + … · 12.2 MB", once finished. */
  summary?: string;
  downloadLink?: string;
  file?: { blob: Blob; name: string };
};

const idle: AnkiExport = { choosing: false, build: emptyBuild, run: 0 };

// Lives outside React: signing up swaps the deck for the account's, which
// remounts every page, and neither a deck being built nor a finished one
// should be lost to that. Keyed by course code (`fra-eng`).
const exports = new Map<string, AnkiExport>();
const listeners = new Set<() => void>();

function subscribe(listener: () => void) {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

function set(course: string, value: AnkiExport) {
  exports.set(course, value);
  listeners.forEach((listener) => listener());
}

/** Begin a new export; the returned updater writes only while this run is
 * the course's latest, so a superseded build can't touch a newer one. */
export function startExport(course: string, owner: string | undefined) {
  const run = (exports.get(course)?.run ?? 0) + 1;
  set(course, { ...idle, owner, run });
  return (change: (current: AnkiExport) => AnkiExport) => {
    const current = exports.get(course);
    if (current?.run === run) set(course, change(current));
  };
}

/** Hand a signed-out visitor's exports to the account they sign into, so no
 * later account on this browser sees them. */
export function claimExports(user: string | undefined) {
  if (!user) return;
  for (const [course, current] of exports) {
    if (current.owner === undefined) set(course, { ...current, owner: user });
  }
}

/** A signed-out visitor's export carries over when they sign up; another
 * account's export on this browser never shows. */
export function useAnkiExport(course: string, user: string | undefined) {
  const stored = useSyncExternalStore(subscribe, () => exports.get(course) ?? idle);
  return stored.owner === undefined || stored.owner === user ? stored : idle;
}
