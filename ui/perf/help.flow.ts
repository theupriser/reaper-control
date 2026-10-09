import { expect, test } from "@playwright/test";
import { tauriStandIn } from "./tauri-stand-in";

const catalog = { revision: 1, setlist_revision: 1, project_id: "project", songs: [], project_songs: [], cues: [], setlists: [], active_setlist: null };
const connected = { status: { Connected: { extension_version: "test" } }, live: null, catalog };

test("Help explains the special markers", async ({ page }) => {
  await page.addInitScript(tauriStandIn, connected);
  await page.goto("/");
  await page.getByRole("button", { name: "Help" }).click();
  await expect(page.getByRole("heading", { name: "Help", level: 1 })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Special markers" })).toBeVisible();
  await expect(page.getByText("!length:45: the song counts as 45 seconds", { exact: false })).toBeVisible();
  await expect(page.getByText("!length:45 !bpm:140 !1008", { exact: true })).toBeVisible();
  await page.screenshot({ path: "perf-results/help.png", fullPage: true });
});
