import { expect, test } from "@playwright/test";
import { tauriStandIn } from "./tauri-stand-in";

const catalog = { revision: 1, setlist_revision: 1, project_id: "project", songs: [], project_songs: [], cues: [], setlists: [], active_setlist: null };
const view = { status: { Connected: { extension_version: "test" } }, live: null, catalog };

test("a person edits the note table and saves it", async ({ page }) => {
  await page.addInitScript(tauriStandIn, view);
  await page.goto("/");
  await page.getByRole("navigation").getByRole("button", { name: "Settings" }).click();

  await expect(page.getByLabel("Note", { exact: true }).first()).toHaveValue("60");
  await page.getByRole("button", { name: "Add a note" }).click();
  await page.getByLabel("Note", { exact: true }).nth(1).fill("61");
  await page.getByLabel("Action", { exact: true }).nth(1).selectOption({ label: "Next song" });
  await page.screenshot({ path: "perf-results/settings.png" });
  await page.getByRole("button", { name: "Save" }).click();

  const saved = await page.evaluate(() => (window as any).__saved);
  expect(saved[0].midi_notes).toEqual([{ note: 60, action: "TogglePlay" }, { note: 61, action: "Next" }]);
});

test("a note used twice is refused with its number and nothing is saved", async ({ page }) => {
  await page.addInitScript(tauriStandIn, view);
  await page.goto("/");
  await page.getByRole("navigation").getByRole("button", { name: "Settings" }).click();
  await page.getByRole("button", { name: "Add a note" }).click();
  await page.getByLabel("Note", { exact: true }).nth(1).fill("60");
  await page.getByRole("button", { name: "Save" }).click();

  await expect(page.getByRole("status")).toContainText("Note 60 is used twice");
  expect(await page.evaluate(() => (window as any).__saved ?? [])).toEqual([]);

  await page.getByLabel("Note", { exact: true }).nth(1).fill("61");
  await page.getByRole("button", { name: "Save" }).click();
  await expect.poll(() => page.evaluate(() => (window as any).__saved?.length ?? 0)).toBe(1);
});

test("a person picks a light theme and large touch targets and they apply at once", async ({ page }) => {
  await page.addInitScript(tauriStandIn, view);
  await page.goto("/");
  await page.getByRole("navigation").getByRole("button", { name: "Settings" }).click();
  await expect(page.locator("html")).toHaveAttribute("data-theme", "dark");

  await page.getByLabel("Theme").selectOption({ label: "Light" });
  await page.getByLabel("Touch targets").selectOption({ label: "Large" });
  await page.getByRole("button", { name: "Save" }).click();

  await expect(page.locator("html")).toHaveAttribute("data-theme", "light");
  await expect(page.locator("html")).toHaveAttribute("data-touch", "large");
  const saved = await page.evaluate(() => (window as any).__saved);
  expect(saved[0].appearance).toEqual({ theme: "light", density: "comfortable", touch: "large" });
  await page.screenshot({ path: "perf-results/appearance.png" });
});
