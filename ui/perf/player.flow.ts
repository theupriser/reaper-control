import { expect, test, type Page } from "@playwright/test";
import { tauriStandIn } from "./tauri-stand-in";

const song = (number: number, start: number, extra = {}) => ({ id: `s${number}`, number, name: `Song ${number}`, start, end: start + 200, colour: null, hard_stop: false, length: null, bpm: 120, ...extra });
const view = {
  status: { Connected: { extension_version: "test" } },
  live: {
    sequence: 1, timestamp: 1, transport: "Playing", position: 210, phase: "Playing", setlist_id: "set", current_song: 1, next_song: 2,
    autoplay: true, count_in: false, record_armed: false, catalog_revision: 1, setlist_revision: 1,
  },
  catalog: {
    revision: 1, setlist_revision: 1, project_id: "project", songs: [song(1, 0), song(2, 200, { bpm: 96 }), song(3, 400, { hard_stop: true })], project_songs: [], cues: [{ id: "c", name: "Chorus", position: 250 }],
    setlists: [{ id: "set", name: "Friday gig", revision: 1, entries: [{ id: 1, song_id: "s1" }, { id: 2, song_id: "s2" }, { id: 3, song_id: "s3" }] }],
    active_setlist: "set",
  },
};
const dispatched = (page: Page) => page.evaluate(() => (window as any).__dispatched ?? []);

test("the Player screen shows the setlist and sends the transport commands", async ({ page }) => {
  await page.addInitScript(tauriStandIn, view);
  await page.goto("/");

  await expect(page.getByRole("heading", { name: "Player" })).toBeVisible();
  await expect(page.getByLabel("Choose the setlist to play")).toHaveValue("set");
  await expect(page.getByText("NOW PLAYING · 2 OF 3")).toBeVisible();
  const list = page.getByRole("list", { name: "Songs in the setlist" });
  await expect(list.getByRole("listitem")).toHaveCount(3);
  await expect(list.getByRole("listitem").nth(1)).toContainText("Song 2");
  await expect(list.getByRole("listitem").nth(2)).toContainText("Next");
  await expect(list.getByRole("listitem").nth(2)).toContainText("HARD STOP");
  await expect(page.getByText("96 BPM").first()).toBeVisible();

  await page.getByRole("button", { name: "Pause" }).click();
  await page.getByRole("button", { name: "Next song" }).click();
  await page.getByRole("switch", { name: "Arm recording" }).click();
  await page.getByRole("switch", { name: "Auto-resume playback" }).click();
  const names = (await dispatched(page)).map((entry: unknown) => JSON.stringify(entry));
  expect(names.join(" ")).toMatch(/Pause.*Next.*ToggleRecordArm.*ToggleAutoResume/);

  await page.getByLabel("Choose the setlist to play").selectOption("");
  expect(JSON.stringify((await dispatched(page)).at(-1))).toContain("SetActiveSetlist");
});

test("the Player layout fits wide and narrow windows without sideways scrolling", async ({ page }) => {
  await page.addInitScript(tauriStandIn, view);
  for (const [width, height] of [[1280, 800], [720, 560]]) {
    await page.setViewportSize({ width, height });
    await page.goto("/");
    await expect(page.getByRole("heading", { name: "Player" })).toBeVisible();
    const overflow = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
    expect(overflow).toBeLessThanOrEqual(0);
    await page.screenshot({ path: `perf-results/player-${width}.png` });
  }
});
