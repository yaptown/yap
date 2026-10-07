import {
  FluentBundle,
  FluentNumber,
  FluentResource,
  type FluentVariable,
} from "@fluent/bundle";
import english from "../../../libraries/yap-i18n/locales/en/yap.ftl?raw";

export type UiLanguage = "en" | "fr" | "es" | "zh-Hans";
export const UI_LANGUAGE_KEY = "yap-ui-language";

export function negotiate(preferences: readonly string[]): UiLanguage {
  for (const tag of preferences) {
    const primary = tag.toLowerCase().split(/[-_]/)[0];
    if (primary === "en" || primary === "fr" || primary === "es")
      return primary;
    if (primary === "zh") return "zh-Hans";
  }
  return "en";
}

function makeBundle(language: UiLanguage, source: string): FluentBundle {
  const bundle = new FluentBundle(language, { useIsolating: false });
  const errors = bundle.addResource(new FluentResource(source));
  if (errors.length)
    throw new Error(`Invalid UI catalog: ${errors.join(", ")}`);
  return bundle;
}

const fallback = makeBundle("en", english);
let active = fallback;
let language: UiLanguage = "en";

export function uiLanguage(): UiLanguage {
  return language;
}

export async function initializeUiLanguage(): Promise<UiLanguage> {
  const override = localStorage.getItem(UI_LANGUAGE_KEY);
  await loadUiLanguage(negotiate(override ? [override] : navigator.languages));
  document.documentElement.lang = language;
  return language;
}

/** Also used by workers, which receive the host's independent UI locale. */
export async function loadUiLanguage(next: UiLanguage): Promise<void> {
  const loaders = {
    fr: () => import("../../../libraries/yap-i18n/locales/fr/yap.ftl?raw"),
    es: () => import("../../../libraries/yap-i18n/locales/es/yap.ftl?raw"),
    "zh-Hans": () =>
      import("../../../libraries/yap-i18n/locales/zh-Hans/yap.ftl?raw"),
  };
  active =
    next === "en"
      ? fallback
      : makeBundle(next, (await loaders[next]()).default);
  language = next;
}

export function t(id: string, args?: Record<string, FluentVariable>): string {
  // Preserve existing ungrouped interpolations; callers retain explicit decimal
  // or grouped formatting by passing strings. Counts remain numeric selectors.
  const variables =
    args &&
    Object.fromEntries(
      Object.entries(args).map(([key, value]) => [
        key,
        typeof value === "number"
          ? new FluentNumber(value, {
              useGrouping: false,
              maximumFractionDigits: 20,
            })
          : value,
      ]),
    );
  const [message, attribute] = id.split(".");
  const english = fallback.getMessage(message);
  const original = attribute ? english?.attributes[attribute] : english?.value;
  if (original == null) throw new Error(`Missing English UI message: ${id}`);
  const translated = active.getMessage(message);
  const pattern = attribute
    ? translated?.attributes[attribute]
    : translated?.value;
  const errors: Error[] = [];
  const value =
    pattern == null
      ? fallback.formatPattern(original, variables, errors)
      : active.formatPattern(pattern, variables, errors);
  if (errors.length)
    throw new Error(`Invalid UI message ${id}: ${errors.join(", ")}`);
  return value;
}
