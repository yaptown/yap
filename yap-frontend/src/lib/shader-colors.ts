// The main thread gets both bands and static fallback from the shared core.
// Workers only receive this data via messages; they never load WASM.
import { background_palette, type BackgroundPalette } from "yap-frontend-rs";

export type ShaderTheme = "dark" | "light" | "oled";

const palettes = new Map<ShaderTheme, BackgroundPalette>();
export function getBackgroundPalette(theme: ShaderTheme): BackgroundPalette {
  let palette = palettes.get(theme);
  if (!palette) {
    palette = background_palette(
      theme === "dark" ? "Dark" : theme === "oled" ? "Oled" : "Light",
    );
    palettes.set(theme, palette);
  }
  return palette;
}

export function getShaderBackgroundCss(theme: ShaderTheme): string {
  const { r, g, b } = getBackgroundPalette(theme).fallback;
  return `rgb(${[r, g, b].map((c) => Math.round(c * 255)).join(", ")})`;
}
