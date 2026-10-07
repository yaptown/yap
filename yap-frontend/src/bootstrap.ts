import "./core/instrument";
import { initializeUiLanguage, t } from "./lib/i18n";

async function boot() {
  const language = await initializeUiLanguage();
  const manifest = document.querySelector<HTMLLinkElement>(
    'link[rel="manifest"]',
  );
  if (manifest && language !== "en") {
    manifest.href = new URL(
      `manifest-${language}.webmanifest`,
      manifest.href,
    ).href;
  }
  const loading = document.getElementById("loading-text")!;
  loading.textContent = t("startup-loading");
  loading.dataset.error = t("startup-trouble");
  document.getElementById("reload-button")!.textContent = t("startup-reload");
  for (const [selector, content] of [
    ['meta[name="description"]', t("app-description")],
    ['meta[property="og:description"]', t("app-social-description")],
    ['meta[name="twitter:description"]', t("app-social-description")],
    ['meta[property="og:image:alt"]', t("app-social-image-description")],
  ])
    document.querySelector(selector)?.setAttribute("content", content);
  // main statically imports WASM; do not evaluate it until the UI locale is ready.
  await import("./main");
}

void boot();
