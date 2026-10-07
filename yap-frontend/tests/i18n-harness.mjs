import * as React from "react";
import * as jsxRuntime from "react/jsx-runtime";
import { readFileSync } from "node:fs";
import vm from "node:vm";
import ts from "typescript";
import * as fluent from "@fluent/bundle";

/** Run the real WASM-free translator in Node, resolving Vite's raw imports. */
export function loadI18n({
  preferences = ["en"],
  override = null,
  catalog,
} = {}) {
  const source = new URL("../src/lib/i18n.ts", import.meta.url);
  const exports = {};
  const document = { documentElement: { lang: "en" } };
  vm.runInNewContext(
    ts.transpileModule(readFileSync(source, "utf8"), {
      compilerOptions: {
        module: ts.ModuleKind.CommonJS,
        target: ts.ScriptTarget.ES2022,
      },
    }).outputText,
    {
      exports,
      document,
      navigator: { languages: preferences },
      localStorage: { getItem: () => override },
      require(specifier) {
        if (specifier === "@fluent/bundle") return fluent;
        if (!specifier.endsWith(".ftl?raw"))
          throw new Error(`Unexpected translator import: ${specifier}`);
        const url = new URL(specifier.replace(/\?raw$/, ""), source);
        return { default: catalog?.(url) ?? readFileSync(url, "utf8") };
      },
    },
  );
  return { ...exports, document };
}

export function loadRich(i18n) {
  const exports = {};
  vm.runInNewContext(
    ts.transpileModule(
      readFileSync(
        new URL("../src/lib/i18n-react.tsx", import.meta.url),
        "utf8",
      ),
      {
        compilerOptions: {
          module: ts.ModuleKind.CommonJS,
          target: ts.ScriptTarget.ES2022,
          jsx: ts.JsxEmit.ReactJSX,
        },
      },
    ).outputText,
    {
      exports,
      require(specifier) {
        if (specifier === "./i18n") return i18n;
        if (specifier === "react") return React;
        if (specifier === "react/jsx-runtime") return jsxRuntime;
        throw new Error(`Unexpected rich-text import: ${specifier}`);
      },
    },
  );
  return exports.rich;
}
