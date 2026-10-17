import { FluentBundle, FluentResource } from "@fluent/bundle";
import fs from "fs";
import path from "path";
import { defineConfig, type Plugin, type Rollup } from "vite";
import react from "@vitejs/plugin-react";
import wasm from "vite-plugin-wasm";
import tailwindcss from "@tailwindcss/vite";
import { VitePWA, type ManifestOptions } from "vite-plugin-pwa";
import { visualizer } from "rollup-plugin-visualizer";
import { sentryVitePlugin } from "@sentry/vite-plugin";

// Share copy with the app, including install metadata before React starts.
const otherUiLanguages = ["fr", "es", "zh-Hans"];
const uiBundles = new Map(
  ["en", ...otherUiLanguages].map((locale) => {
    const bundle = new FluentBundle(locale, { useIsolating: false });
    bundle.addResource(
      new FluentResource(
        fs.readFileSync(
          path.resolve(
            __dirname,
            `../libraries/yap-i18n/locales/${locale}/yap.ftl`,
          ),
          "utf8",
        ),
      ),
    );
    return [locale, bundle] as const;
  }),
);
function uiText(id: string, locale = "en"): string {
  const english = uiBundles.get("en")!;
  const active = uiBundles.get(locale)!;
  const pattern = active.getMessage(id)?.value;
  return pattern == null
    ? english.formatPattern(english.getMessage(id)!.value!)
    : active.formatPattern(pattern);
}

function manifest(locale = "en"): Partial<ManifestOptions> {
  return {
    name: "Yap.Town",
    short_name: "Yap",
    description: uiText("app-description", locale),
    theme_color: "#0A0A0A",
    background_color: "#0A0A0A",
    id: "https://yap.town/",
    start_url: "/",
    icons: [
      {
        src: "pwa-64x64.png",
        sizes: "64x64",
        type: "image/png",
      },
      {
        src: "pwa-192x192.png",
        sizes: "192x192",
        type: "image/png",
      },
      {
        src: "pwa-512x512.png",
        sizes: "512x512",
        type: "image/png",
      },
      {
        src: "maskable-icon-512x512.png",
        sizes: "512x512",
        type: "image/png",
        purpose: "maskable",
      },
    ],
    screenshots: [
      {
        src: "screenshot-wide.png",
        sizes: "1988x1176",
        type: "image/gif",
        form_factor: "wide",
        label: uiText("app-screenshot", locale),
      },
      {
        src: "screenshot-mobile.png",
        sizes: "584x1260",
        type: "image/gif",
        label: uiText("app-screenshot", locale),
      },
    ],
    shortcuts: [
      {
        name: uiText("common-dictionary", locale),
        url: "/dictionary",
        description: uiText("app-shortcut-dictionary-description", locale),
      },
    ],
  };
}

function localizedManifest(
  base: Partial<ManifestOptions>,
  locale: string,
): Partial<ManifestOptions> {
  const copy = manifest(locale);
  return {
    ...base,
    lang: locale,
    description: copy.description,
    screenshots: base.screenshots?.map((shot, index) => ({
      ...shot,
      label: copy.screenshots![index].label,
    })),
    shortcuts: base.shortcuts?.map((shortcut, index) => ({
      ...shortcut,
      name: copy.shortcuts![index].name,
      description: copy.shortcuts![index].description,
    })),
  };
}

function localizedManifestsPlugin(): Plugin {
  return {
    name: "localized-install-manifests",
    configureServer(server) {
      server.middlewares.use((req, res, next) => {
        const match = req.url?.match(
          /^\/manifest-(fr|es|zh-Hans)\.webmanifest(?:\?.*)?$/,
        );
        if (!match) return next();
        res.setHeader("Content-Type", "application/manifest+json");
        res.end(
          JSON.stringify(
            localizedManifest(
              { display: "standalone", ...manifest() },
              match[1],
            ),
          ),
        );
      });
    },
    generateBundle: {
      order: "post",
      handler(_options, bundle) {
        const base = bundle["manifest.webmanifest"];
        if (!base || base.type !== "asset")
          throw new Error("PWA manifest was not emitted");
        const manifest = JSON.parse(
          typeof base.source === "string"
            ? base.source
            : Buffer.from(base.source).toString("utf8"),
        ) as Partial<ManifestOptions>;
        for (const locale of otherUiLanguages)
          this.emitFile({
            type: "asset",
            fileName: `manifest-${locale}.webmanifest`,
            source: JSON.stringify(localizedManifest(manifest, locale)),
          });
      },
    },
  };
}

function uiFallbackPlugin(): Plugin {
  return {
    name: "ui-english-fallback",
    transformIndexHtml(html) {
      return html.replace(/%UI_([a-z0-9-]+)%/g, (_, id: string) =>
        uiText(id)
          .replaceAll("&", "&amp;")
          .replaceAll('"', "&quot;")
          .replaceAll("<", "&lt;"),
      );
    },
  };
}

// Serve static site pages (dictionary, blog) without SPA fallback intercepting
function staticSitePlugin(): Plugin {
  return {
    name: "static-site",
    configureServer(server) {
      server.middlewares.use((req, _res, next) => {
        if (
          req.url?.startsWith("/d/") ||
          req.url === "/d" ||
          req.url?.startsWith("/blog/") ||
          req.url === "/blog"
        ) {
          // Rewrite directory requests to their index.html
          if (!req.url.includes(".")) {
            const path = req.url.endsWith("/") ? req.url : req.url + "/";
            // eslint-disable-next-line no-param-reassign -- Connect middleware rewrites the URL before static serving.
            req.url = path + "index.html";
          }
        }
        next();
      });
    },
  };
}

// Captured fixtures are development inputs, never production assets.
function screenFixturesPlugin(): Plugin {
  const root = path.resolve(__dirname, "../fixtures");
  return {
    name: "screen-fixtures",
    apply: "serve",
    configureServer(server) {
      server.middlewares.use((req, res, next) => {
        if (req.method !== "GET" || !req.url?.startsWith("/__fixtures/"))
          return next();
        const directories = fs
          .readdirSync(root, { withFileTypes: true })
          .filter((entry) => entry.isDirectory())
          .map((entry) => path.join(root, entry.name));
        const files = new Map<string, string>();
        for (const directory of directories) {
          for (const entry of fs.readdirSync(directory, {
            withFileTypes: true,
          })) {
            if (!entry.isFile() || !entry.name.endsWith(".json")) continue;
            if (files.has(entry.name))
              throw new Error(`Duplicate fixture name: ${entry.name}`);
            files.set(entry.name, path.join(directory, entry.name));
          }
        }
        const names = [...files.keys()];
        const name = req.url.slice("/__fixtures/".length);
        res.setHeader("Content-Type", "application/json");
        if (name === "index.json")
          res.end(
            JSON.stringify(names.map((name) => name.slice(0, -5)).sort()),
          );
        else if (names.includes(name))
          res.end(fs.readFileSync(files.get(name)!));
        else {
          res.writeHead(404);
          res.end(JSON.stringify({ error: "Unknown fixture" }));
        }
      });
    },
  };
}

// A WASM built with `--features local-backend` hardcodes http://localhost:21516
// as the AI backend. `pnpm build` bundles whatever sits in yap-frontend-rs/pkg,
// so a stale local-test build can silently ship to production (this happened:
// autograde/TTS were broken on yap.town for three days). Fail the build instead.
function localBackendGuardPlugin() {
  return {
    name: "local-backend-guard",
    apply: "build" as const,
    buildStart() {
      if (process.env.ALLOW_LOCAL_BACKEND) return;
      const wasmPath = path.resolve(
        __dirname,
        "../yap-frontend-rs/pkg/yap_frontend_rs_bg.wasm",
      );
      if (fs.readFileSync(wasmPath).includes("localhost:21516")) {
        throw new Error(
          "yap-frontend-rs/pkg was built with --features local-backend (AI backend = localhost:21516). " +
            "Rebuild it without the feature (cd yap-frontend-rs && CARGO_PROFILE_RELEASE_LTO=true wasm-pack build --release) " +
            "or set ALLOW_LOCAL_BACKEND=1 to build anyway.",
        );
      }
    },
  };
}

// Boot is a chain: bootstrap → main → WASM → App. Without hints the browser
// only discovers each file once the previous one has run, so the WASM and the
// App chunk download one after the other. Preloading fetches both up front
// without evaluating them (App must not run before the UI locale is set).
function bootPreloadPlugin(): Plugin {
  return {
    name: "boot-preload",
    apply: "build",
    transformIndexHtml: {
      order: "post",
      handler(_html, { bundle }) {
        const files = Object.values(bundle!);
        const wasm = files.find((f) =>
          /yap_frontend_rs_bg-.*\.wasm$/.test(f.fileName),
        )!;
        const app = files.find(
          (f) =>
            f.type === "chunk" &&
            f.moduleIds.some((id) => id.endsWith("/src/app/App.tsx")),
        ) as Rollup.OutputChunk;
        return [
          {
            tag: "link",
            attrs: {
              rel: "preload",
              as: "fetch",
              crossorigin: "",
              href: `/${wasm.fileName}`,
            },
            injectTo: "head",
          },
          ...[app.fileName, ...app.imports].map((file) => ({
            tag: "link",
            attrs: { rel: "modulepreload", crossorigin: "", href: `/${file}` },
            injectTo: "head" as const,
          })),
        ];
      },
    },
  };
}

// https://vite.dev/config/
export default defineConfig({
  worker: { format: "es" },
  build: {
    sourcemap: true,
  },
  server: {
    fs: { allow: [path.resolve(__dirname, "..")] },
    // Allow access through the yap-preview.beef.baby reverse proxy
    allowedHosts: ["yap-preview.beef.baby"],
  },
  plugins: [
    uiFallbackPlugin(),
    localizedManifestsPlugin(),
    localBackendGuardPlugin(),
    bootPreloadPlugin(),
    staticSitePlugin(),
    screenFixturesPlugin(),
    VitePWA({
      registerType: "autoUpdate",
      devOptions: {
        enabled: false,
        //enabled: true,
        type: "module",
      },
      workbox: {
        globPatterns: [
          "**/*.{js,css,html,ico,png,svg,wasm,wav,mp3}",
          "manifest-*.webmanifest",
        ],
        globIgnores: ["**/d/**", "**/blog/**", "**/sql-wasm*.wasm"],
        importScripts: [],
        // The WASM bundle must be precached for offline use, and it grows: it
        // crossed the old 4 MiB cap in #208, which failed the deploy build.
        maximumFileSizeToCacheInBytes: 8 * 1024 * 1024,
        // Match the bare path as well as the trailing-slash form: /blog and /d
        // are real server-rendered pages, and without the $ alternative the
        // service worker served the SPA shell for them, which 404s.
        navigateFallbackDenylist: [
          /^\/d(\/|$)/,
          /^\/blog(\/|$)/,
          /^\/sitemap/,
          /^\/robots\.txt/,
        ],
      },
      manifest: manifest(),
    }),
    react({
      babel: {
        plugins: [
          [
            "babel-plugin-react-compiler",
            {
              compilationMode: "infer", // Only compile components that would benefit
              runtimeModule: "react",
            },
          ],
        ],
      },
    }),
    wasm(),
    tailwindcss(),
    visualizer({
      open: false, // Don't auto-open on every build
      filename: "bundle-analysis.html",
      gzipSize: true,
      brotliSize: true,
    }),
    sentryVitePlugin({
      org: "yaptown",
      project: "javascript-react",
      authToken: process.env.SENTRY_AUTH_TOKEN,
    }),
  ],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
});
