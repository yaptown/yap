import { test, expect } from "@playwright/test";
import path from "node:path";
import { fakeLogin, seedFrenchDeck } from "./helpers";

test("capture challenge fixtures", async ({ page, request }) => {
  const response = await request.get("/__fixtures/index.json");
  expect(response.ok()).toBeTruthy();
  const names: string[] = await response.json();
  const only = (process.env.YAP_FIXTURE ?? "").split(",").filter(Boolean);
  const selected = names.filter(
    (name) => only.length === 0 || only.some((prefix) => name.startsWith(prefix)),
  );
  expect(selected.length).toBeGreaterThan(0);
  test.setTimeout(120_000 + selected.length * 10_000);
  await fakeLogin(page);
  await page.goto("/");
  await seedFrenchDeck(page);
  await page.waitForTimeout(4000); // flush deck selection to OPFS before navigation
  for (const name of selected) {
    await page.goto(`/fixture/${name}`);
    await expect(
      page.locator(`[data-fixture-rendered="${name}"]`),
    ).toBeVisible();
    await page.waitForTimeout(2500);
    await page.screenshot({
      path: path.resolve("screenshots-out/fixtures", `${name}-web.png`),
    });
  }
});
