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
    if (name === "pronunciation") {
      const highlight = page.locator('span[class~="bg-caution/30"]').first();
      await expect(highlight).toBeVisible();
      await expect(highlight).toHaveCSS("padding-left", "0px");
      await expect(highlight).toHaveCSS("padding-right", "0px");
    }
    if (name === "translation-elision") {
      const chunk = page.locator("h2 .whitespace-nowrap").filter({ hasText: "qu'il" });
      await expect(chunk).toHaveCount(1);
      await expect(chunk).toHaveCSS("white-space", "nowrap");
      const apostrophe = await chunk.getByText("qu'", { exact: true }).boundingBox();
      const pronoun = await chunk.getByText("il", { exact: true }).boundingBox();
      expect(apostrophe).not.toBeNull();
      expect(pronoun).not.toBeNull();
      expect(apostrophe!.y).toBeCloseTo(pronoun!.y, 0);
    }
    await page.waitForTimeout(2500);
    await page.screenshot({
      path: path.resolve("screenshots-out/fixtures", `${name}-web.png`),
    });
    if (name === "dictation-empty") {
      // A long answer must wrap inside the field rather than push its
      // unbreakable run (field + attached ".") past the sentence's edge.
      const field = page.locator("textarea").first();
      await field.fill("Pas pour ta vie, ".repeat(8));
      const chunk = field.locator(
        "xpath=ancestor::span[contains(@class,'whitespace-nowrap')][1]",
      );
      const run = await chunk.boundingBox();
      const line = await chunk.locator("xpath=..").boundingBox();
      expect(run!.x + run!.width).toBeLessThanOrEqual(line!.x + line!.width + 0.5);
    }
  }
});
