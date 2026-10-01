// Web-specific locale aliases, joined with shared language metadata
// from Rust. After changing learning_metadata.rs, rebuild WASM and run
// `pnpm generate:metadata` in yap-frontend; both builds check for stale metadata.
// This module has no WASM runtime dependency, so the MCP widget can share it.
import { LANGUAGE_METADATA } from "./learning-metadata.generated";
import type { Language, LanguageMetadata } from "../../../yap-frontend-rs/pkg";

export interface LanguageMeta extends LanguageMetadata {
  /** Browser locale aliases, lowercased. Bare zh, es, and pt default to the inheriting course. */
  browserCodes: string[];
}

export const LANGUAGES: Record<Language, LanguageMeta> = {
  English: {
    ...LANGUAGE_METADATA.English,
    browserCodes: ["en"],
  },
  French: {
    ...LANGUAGE_METADATA.French,
    browserCodes: ["fr"],
  },
  SpanishLatinAmerican: {
    ...LANGUAGE_METADATA.SpanishLatinAmerican,
    browserCodes: ["es", "es-419", "es-mx", "es-us"],
  },
  SpanishPeninsular: {
    ...LANGUAGE_METADATA.SpanishPeninsular,
    browserCodes: ["es-es"],
  },
  German: {
    ...LANGUAGE_METADATA.German,
    browserCodes: ["de"],
  },
  Italian: {
    ...LANGUAGE_METADATA.Italian,
    browserCodes: ["it"],
  },
  PortugueseBrazilian: {
    ...LANGUAGE_METADATA.PortugueseBrazilian,
    browserCodes: ["pt", "pt-br"],
  },
  PortugueseEuropean: {
    ...LANGUAGE_METADATA.PortugueseEuropean,
    browserCodes: ["pt-pt"],
  },
  Russian: {
    ...LANGUAGE_METADATA.Russian,
    browserCodes: ["ru"],
  },
  Korean: {
    ...LANGUAGE_METADATA.Korean,
    browserCodes: ["ko"],
  },
  Japanese: {
    ...LANGUAGE_METADATA.Japanese,
    browserCodes: ["ja"],
  },
  ChineseSimplified: {
    ...LANGUAGE_METADATA.ChineseSimplified,
    browserCodes: ["zh", "zh-hans", "zh-cn", "zh-sg"],
  },
  ChineseTraditional: {
    ...LANGUAGE_METADATA.ChineseTraditional,
    browserCodes: ["zh-hant", "zh-tw", "zh-hk", "zh-mo"],
  },
  Hindi: {
    ...LANGUAGE_METADATA.Hindi,
    browserCodes: ["hi"],
  },
  Thai: {
    ...LANGUAGE_METADATA.Thai,
    browserCodes: ["th"],
  },
};

/** Every language, in the order the table declares them. */
export const ALL_LANGUAGES = Object.keys(LANGUAGES) as Language[];

/**
 * Project one field out of the table into its own lookup. The cast is safe —
 * and lives here alone — because the source keys are exactly `Language`.
 */
export function mapLanguages<T>(
  select: (meta: LanguageMeta) => T,
): Record<Language, T> {
  return Object.fromEntries(
    ALL_LANGUAGES.map((language) => [language, select(LANGUAGES[language])]),
  ) as Record<Language, T>;
}

const BY_ISO_CODE: Record<string, Language> = Object.fromEntries(
  ALL_LANGUAGES.map((language) => [LANGUAGES[language].isoCode, language]),
);

const BY_BROWSER_CODE: Record<string, Language> = Object.fromEntries(
  ALL_LANGUAGES.flatMap((language) =>
    LANGUAGES[language].browserCodes.map((code) => [code, language]),
  ),
);

/** Inverse of `LanguageMeta.isoCode`; null for a code we don't teach. */
export function isoCodeToLanguage(isoCode: string): Language | null {
  return BY_ISO_CODE[isoCode] ?? null;
}

/** True when the string is one of the `Language` enum's variant names. */
export function isLanguage(value: string): value is Language {
  return value in LANGUAGES;
}

/**
 * The `Language` implied by the browser's locale, or null if we don't teach
 * it. The full tag wins over the base subtag so zh-TW picks traditional
 * rather than falling through to simplified.
 */
export function detectBrowserLanguage(): Language | null {
  const browserLang = navigator.language || navigator.languages?.[0];
  if (!browserLang) return null;
  const tag = browserLang.toLowerCase();
  return BY_BROWSER_CODE[tag] ?? BY_BROWSER_CODE[tag.split("-")[0]] ?? null;
}
