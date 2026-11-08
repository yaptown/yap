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
    if (fixture?.type === "Challenge") {
      const draft = fixture.view.draft;
      if (draft?.type === "Translation") {
        await expect(page.locator("textarea").first()).toHaveValue(draft.text);
      } else if (draft?.type === "Transcription" && draft.inputs.length === 1) {
        await expect(page.locator("textarea").first()).toHaveValue(draft.inputs[0].text);
      }
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
    if (name === "idle-all-caught-up" && fixture?.type === "Idle" && fixture.view.type === "Idle") {
      await expect(page.getByRole("menuitem")).toHaveCount(0);
      await page.getByRole("button", { name: fixture.view.manual_add_heading, exact: true }).click();
      await expect(page.getByRole("menuitem").first()).toBeVisible();
      await page.keyboard.press("Escape");
    }
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

test("typed drafts preserve focus, accents, Enter submission and cancellation", async ({ page }) => {
  await page.setViewportSize({ width: 1100, height: 900 });
  await fakeLogin(page);
  await page.goto("/");
  await seedFrenchDeck(page);
  await page.waitForTimeout(4000);

  for (const name of ["translation-empty", "dictation-empty"]) {
    let pending: import("@playwright/test").Route | undefined;
    await page.route("**/autograde-*", (route) => { pending = route; });
    await page.goto(`/fixture/${name}`);
    await expect(page.locator(`[data-fixture-rendered="${name}"]`)).toBeVisible();
    const input = page.locator("textarea").first();
    await expect(input).toBeFocused();
    await input.fill("ab");
    let answer = "ab";
    if (name === "dictation-empty") {
      await input.evaluate((element: HTMLTextAreaElement) => element.setSelectionRange(0, 0));
      await page.getByRole("button", { name: "é", exact: true }).click();
      await expect(input).toHaveValue("éab");
      await expect(input).toBeFocused();
      await page.keyboard.down("Shift");
      await page.getByRole("button", { name: "É", exact: true }).click();
      await page.keyboard.up("Shift");
      answer = "éÉab";
      await expect(input).toHaveValue(answer);
    }
    await input.press("Enter");
    await expect.poll(() => pending !== undefined).toBe(true);
    // Hold the grading response so cancellation is deterministic and no
    // fixture answer is sent to the real autograder.
    await page.locator(".sticky button").last().click();
    await expect(input).toBeEnabled();
    await expect(input).toHaveValue(answer);
    await pending!.abort();
    await page.unroute("**/autograde-*");
  }
});
