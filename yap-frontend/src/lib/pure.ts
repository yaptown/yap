import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";
import type { Language, VoiceActorInfo } from "../../../yap-frontend-rs/pkg";
import {
  LANGUAGES,
  isLanguage,
  isoCodeToLanguage,
  mapLanguages,
} from "./languages";

export interface PlaybackOptions {
  temporary?: boolean;
  onAudioElement?: (audio: HTMLAudioElement) => void | Promise<void>;
  signal?: AbortSignal;
  onVoiceActor?: (info: VoiceActorInfo) => void;
}

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

// One media element plays at a time, app-wide. TTS playback and the movie
// clip player both register their interrupt here; starting either one stops
// whatever else was playing. Lives here rather than in utils.ts because the
// MCP widget swaps utils.ts for a WASM-free shim that re-exports this module
// — the clip player must reach the registry through that shim too.
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

export const languageFlags = mapLanguages((meta) => meta.flag);

export const nativeLanguageNames = mapLanguages((meta) => meta.nativeName);

// Accepts either a `Language` variant name or a pipeline ISO code, since
// server-side stats hand back the latter.
export function getLanguageFlag(isoCodeOrLanguage: string): string {
  const language = isLanguage(isoCodeOrLanguage)
    ? isoCodeOrLanguage
    : isoCodeToLanguage(isoCodeOrLanguage);
  return language ? LANGUAGES[language].flag : "🌐";
}

export function getLanguageName(isoCodeOrLanguage: string): string {
  const language = isLanguage(isoCodeOrLanguage)
    ? isoCodeOrLanguage
    : isoCodeToLanguage(isoCodeOrLanguage);
  return language ? LANGUAGES[language].nativeName : isoCodeOrLanguage;
}

/**
 * The canonical ISO 639-1 code — use this for every comparison and lookup.
 * Identical to Rust's `Language::iso_639_1`, so both Chinese variants give
 * "zh", which is what movie and book metadata's `original_language` holds.
 */
export function languageToIso6391(language: Language): string {
  return LANGUAGES[language].iso6391;
}

/**
 * Value for an HTML `lang` attribute, and nothing else. Appends the script
 * subtag where one is needed, because `lang` drives Han glyph selection:
 * Traditional Chinese tagged as bare "zh" can render with mainland glyph
 * forms. Never compare against this — see `languageToIso6391`.
 */
export function languageToLangAttr(language: Language): string {
  const { iso6391, script } = LANGUAGES[language];
  return script ? `${iso6391}-${script}` : iso6391;
}
