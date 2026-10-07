import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import assert from "node:assert/strict";
import test from "node:test";
import { loadI18n, loadRich } from "./i18n-harness.mjs";

test("browser preferences and the local override are independent of course languages", async () => {
  for (const [preferences, override, expected] of [
    [["de", "fr-CA", "en"], null, "fr"],
    [["en-GB", "fr"], null, "en"],
    [["zh-Hant-TW"], null, "zh-Hans"],
    [["ja"], null, "en"],
    [["fr"], "es", "es"],
  ]) {
    const i18n = loadI18n({ preferences, override });
    assert.equal(await i18n.initializeUiLanguage(), expected);
    assert.equal(i18n.document.documentElement.lang, expected);
  }
});

test("English preserves whitespace, interpolation, and ungrouped counts", () => {
  const { t } = loadI18n();
  assert.equal(t("names-and"), " and ");
  assert.equal(t("review-cards", { count: 1 }), "Review 1 card");
  assert.equal(t("review-cards", { count: 1000 }), "Review 1000 cards");
  assert.equal(
    t("dictionary-added-word", { word: "chat" }),
    'Added "chat" to your deck',
  );
});

test("missing translations fall back to English; missing English fails loudly", async () => {
  const i18n = loadI18n({
    preferences: ["fr"],
    catalog: (url) =>
      url.pathname.includes("/fr/") ? "only-french = Texte" : undefined,
  });
  await i18n.initializeUiLanguage();
  assert.equal(i18n.t("common-continue"), "Continue");
  assert.throws(
    () => i18n.t("intentionally-missing"),
    /Missing English UI message/,
  );
});

test("whole date messages preserve the original separator spaces", () => {
  const { t } = loadI18n();
  assert.equal(
    t("settings-passkey-dates", { date: "1/2/2026", last_used: "2/3/2026" }),
    "Added 1/2/2026 · Last used 2/3/2026",
  );
});

test("rich messages reorder actual React nodes without HTML interpretation", async () => {
  const i18n = loadI18n({
    preferences: ["fr"],
    catalog: (url) =>
      url.pathname.includes("/fr/")
        ? "about-author-credit = { $author } — <b>credits</b>"
        : undefined,
  });
  await i18n.initializeUiLanguage();
  const rich = loadRich(i18n);
  const result = rich("about-author-credit", {
    author: createElement("a", { href: "/author" }, "Andre"),
  });
  assert.equal(
    renderToStaticMarkup(createElement("span", null, result)),
    '<span><a href="/author">Andre</a> — &lt;b&gt;credits&lt;/b&gt;</span>',
  );
});
