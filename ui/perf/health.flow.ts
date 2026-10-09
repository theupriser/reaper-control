import { expect, test } from "@playwright/test";
import { tauriStandIn } from "./tauri-stand-in";

const connected = { Connected: { extension_version: "test" } };
const catalog = { revision: 1, setlist_revision: 1, project_id: "project", songs: [], project_songs: [], cues: [], setlists: [], active_setlist: null };
const view = (status: unknown) => ({ status, live: null, catalog });

test("a working link shows no banner", async ({ page }) => {
  await page.addInitScript(tauriStandIn, view(connected));
  await page.goto("/");
  await expect(page.getByRole("navigation").getByText("Connected")).toBeVisible();
  await expect(page.getByRole("alert")).toHaveCount(0);
});

test("a link that is down shows a banner and the dialog shows the numbers", async ({ page }) => {
  await page.addInitScript(tauriStandIn, view("NotRunning"));
  await page.goto("/");
  await expect(page.getByRole("alert")).toContainText("REAPER not running");
  await expect(page.getByRole("alert")).toContainText("Open REAPER");

  await page.getByRole("button", { name: "Connection details" }).click();
  const dialog = page.getByRole("dialog", { name: "Connection and health" });
  await expect(dialog).toContainText("Machine CPU");
  await expect(dialog).toContainText("12%");
  await expect(dialog).toContainText("4000 MB / 16000 MB");
  await expect(dialog).toContainText("600 MB");
  await page.screenshot({ path: "perf-results/health.png" });
  await dialog.getByRole("button", { name: "Close" }).click();
  await expect(dialog).toBeHidden();

  const overflow = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
  expect(overflow).toBeLessThanOrEqual(0);
});

test("a problem the app found replaces the banner text and clears again", async ({ page }) => {
  await page.addInitScript(tauriStandIn, view(connected));
  await page.goto("/");
  await page.getByRole("navigation").waitFor();
  await page.evaluate(() => (window as any).__pushEvent("link-problem", "The extension is not loaded"));
  await expect(page.getByRole("alert")).toContainText("The extension is not loaded");
  await page.evaluate(() => (window as any).__pushEvent("link-problem", null));
  await expect(page.getByRole("alert")).toHaveCount(0);
});
