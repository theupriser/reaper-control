import { expect, test } from "@playwright/test";
import { tauriStandIn } from "./tauri-stand-in";

const song = (number: number, start: number) => ({ id: `s${number}`, number, name: `Song ${number}`, start, end: start + 200, colour: null, hard_stop: false, length: null, bpm: 120 });
const view = {
  status: { Connected: { extension_version: "test" } },
  live: {
    sequence: 1, timestamp: 1, transport: "Playing", position: 10, phase: "Playing", setlist_id: "set", current_song: 0, next_song: 1,
    autoplay: true, count_in: false, record_armed: false, catalog_revision: 1, setlist_revision: 1,
  },
  catalog: {
    revision: 1, setlist_revision: 1, project_id: "project", songs: [song(1, 0), song(2, 200)], project_songs: [], cues: [],
    setlists: [{ id: "set", name: "Set", revision: 1, entries: [{ id: 1, song_id: "s1" }, { id: 2, song_id: "s2" }] }],
    active_setlist: "set",
  },
};
const dispatched = (page: import("@playwright/test").Page) => page.evaluate(() => (window as any).__dispatched ?? []);

test("the locked Performer screen ignores stray touches and needs two taps to open", async ({ page }) => {
  await page.addInitScript(tauriStandIn, view);
  await page.goto("/");
  await page.getByRole("button", { name: "Performer mode" }).click();

  await page.getByRole("button", { name: "Lock screen" }).click();
  await expect(page.getByRole("button", { name: "Exit Performer Mode" })).toHaveCount(0);
  await expect(page.getByRole("button", { name: /Auto-resume/ })).toBeDisabled();
  await expect(page.getByRole("button", { name: "Toggle recording" })).toBeDisabled();

  await page.keyboard.press("Escape");
  await page.keyboard.press("a");
  await expect(page.getByRole("button", { name: "Unlock" })).toBeVisible();
  expect(await dispatched(page)).toEqual([]);
  await page.screenshot({ path: "perf-results/stage-lock.png" });

  await page.getByRole("button", { name: "Unlock" }).click();
  await expect(page.getByRole("button", { name: "Tap again to unlock" })).toBeVisible();
  await page.getByRole("button", { name: "Tap again to unlock" }).click();
  await expect(page.getByRole("button", { name: "Exit Performer Mode" })).toBeVisible();
  await expect(page.getByRole("button", { name: /Auto-resume/ })).toBeEnabled();
});

test("an armed unlock that is not confirmed locks again", async ({ page }) => {
  await page.addInitScript(tauriStandIn, view);
  await page.goto("/");
  await page.getByRole("button", { name: "Performer mode" }).click();
  await page.getByRole("button", { name: "Lock screen" }).click();
  await page.getByRole("button", { name: "Unlock" }).click();
  await expect(page.getByRole("button", { name: "Tap again to unlock" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Unlock", exact: true })).toBeVisible({ timeout: 5000 });
});
