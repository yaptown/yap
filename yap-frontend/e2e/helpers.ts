import type { Page } from "@playwright/test";

// Fake a logged-in-but-offline user so routes render without a Supabase
// session, and skip the OPFS capability probe. Runs before any app code.
export async function fakeLogin(page: Page) {
  const loggedOut = process.env.LOGGED_OUT === "1";
  await page.addInitScript((loggedOut: boolean) => {
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
  }, loggedOut);
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
