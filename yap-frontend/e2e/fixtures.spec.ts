import { test, expect } from "@playwright/test";
import path from "node:path";
import { fakeLogin, seedFrenchDeck } from "./helpers";
import type { Fixture, IdleKind } from "../../yap-frontend-rs/pkg";

test("capture screen and challenge fixtures", async ({ page, request }) => {
  const response = await request.get("/__fixtures/index.json");
  expect(response.ok()).toBeTruthy();
  const names: string[] = await response.json();
  const only = (process.env.YAP_FIXTURE ?? "").split(",").filter(Boolean);
  const selected = names.filter(
    (name) =>
      only.length === 0 || only.some((prefix) => name.startsWith(prefix)),
  );
  expect(selected.length).toBeGreaterThan(0);
  test.setTimeout(120_000 + selected.length * 10_000);
  await fakeLogin(page);
  await page.goto("/");
  await seedFrenchDeck(page);
  await page.waitForTimeout(4000); // flush deck selection to OPFS before navigation
  const idleKinds: Record<string, IdleKind> = {
    "idle-first-run": "FirstRun",
    "idle-needs-more-cards": "NeedsMoreCards",
    "idle-all-caught-up": "AllCaughtUp",
  };
  for (const name of selected) {
    const captureResponse = await request.get(`/__fixtures/${name}.json`);
    expect(captureResponse.ok()).toBeTruthy();
    const capture = (await captureResponse.json()) as Fixture;
    const fixture = capture.screen === "Review" ? capture.view.step : undefined;
    if (fixture && name in idleKinds) {
      expect(fixture?.type).toBe("Idle");
      if (fixture?.type === "Idle") {
        expect(fixture.view.type).toBe("Idle");
        if (fixture.view.type === "Idle")
          expect(fixture.view.kind).toBe(idleKinds[name]);
      }
    }
    await page.goto(`/fixture/${name}`);
    await expect(
      page.locator(`[data-fixture-rendered="${name}"]`),
    ).toBeVisible();
    if (fixture?.type === "Idle" && fixture.view.type === "Idle") {
      await expect(
        page.getByText(fixture.view.title, { exact: true }),
      ).toBeVisible();
    }
    if (fixture?.type === "Accomplishment") {
      await expect(
        page.getByRole("heading", { name: /Goal Reached!/ }),
      ).toBeVisible();
    }
    await page.waitForTimeout(2500);
    await page.screenshot({
      path: path.resolve("screenshots-out/fixtures", `${name}-web.png`),
    });
  }
});
