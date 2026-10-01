import type { Page } from "@playwright/test";

// Fake a logged-in-but-offline user so routes render without a Supabase
// session, and skip the OPFS capability probe. Runs before any app code.
export async function fakeLogin(page: Page) {
  const loggedOut = process.env.LOGGED_OUT === "1";
  // Pin the appearance instead of inheriting ThemeProvider's "dark" default.
  // iOS takes its theme from the simulator's appearance, so leaving this to
  // each host's own default makes every parity pair differ by theme —
  // drowning out the differences the comparison is for. `cargo xtask parity
  // --theme` sets this and the simulator's appearance together.
  const theme = process.env.YAP_THEME === "dark" ? "dark" : "light";
  await page.addInitScript(
    ({ loggedOut, theme }: { loggedOut: boolean; theme: string }) => {
      if (!loggedOut) {
        localStorage.setItem(
          "yap-user-info",
          JSON.stringify({
            id: "e2e-screenshot-user",
            email: "screenshots@example.com",
            displayName: "Screenshot Bot",
          }),
        );
      }
      localStorage.setItem("opfs-test-passed", "true");
      localStorage.setItem("yap-pimsleur-acknowledged", "true");
      localStorage.setItem("vite-ui-theme", theme);
    },
    { loggedOut, theme },
  );
}

// Wait for the dev-only window.__weapon hook, then seed French (target) /
// English (native). Mirrors LanguageSelector's onLanguagesConfirmed.
export async function seedFrenchDeck(page: Page) {
  await page.waitForFunction(() => "__weapon" in window, null, {
    timeout: 30_000,
  });
  await page.evaluate(() => {
    const w = (
      window as unknown as {
        __weapon: { add_deck_selection_event: (event: unknown) => void };
      }
    ).__weapon;
    w.add_deck_selection_event({
      SelectBothLanguages: { native: "English", target: "French" },
    });
  });
}
