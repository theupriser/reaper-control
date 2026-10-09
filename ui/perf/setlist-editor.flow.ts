import { expect, test } from "@playwright/test";
import { tauriStandIn } from "./tauri-stand-in";

const song = (number: number, name: string) => ({ id: `s${number}`, number, name, start: number * 100, end: number * 100 + 90, colour: null, hard_stop: false, length: null, bpm: 120 });
const projectSongs = [song(1, "Opener"), song(2, "Ballad"), song(3, "Closer")];
const view = {
  status: { Connected: { extension_version: "test" } },
  live: null,
  catalog: {
    revision: 1, setlist_revision: 4, project_id: "project", songs: projectSongs, project_songs: projectSongs, cues: [],
    setlists: [
      { id: "sat", name: "Saturday", revision: 2, entries: [{ id: 1, song_id: "s1" }, { id: 2, song_id: "s2" }, { id: 3, song_id: "gone" }] },
      { id: "sun", name: "Sunday", revision: 1, entries: [{ id: 1, song_id: "s3" }] },
    ],
    active_setlist: "sun",
  },
};

const dispatched = (page: import("@playwright/test").Page) => page.evaluate(() => (window as any).__dispatched ?? []);

test("edit a setlist, save it and play it", async ({ page }) => {
  await page.addInitScript(tauriStandIn, view);
  await page.goto("/");
  await page.getByRole("navigation").getByRole("button", { name: "Setlists" }).click();

  await expect(page.getByRole("button", { name: /Saturday/ })).toBeVisible();
  await expect(page.getByRole("button", { name: /Sunday/ })).toContainText("Playing");

  await page.getByRole("button", { name: /Saturday/ }).click();
  await expect(page.getByText("Not in the project")).toBeVisible();
  await expect(page.getByRole("button", { name: "Save setlist" })).toBeDisabled();

  await page.getByRole("button", { name: "Move Ballad up" }).click();
  await page.getByRole("button", { name: "Remove gone" }).click();
  await page.getByLabel("Add a song").selectOption("s3");
  await page.getByRole("button", { name: "Add", exact: true }).click();
  await page.getByLabel("Name").fill("Saturday late");
  await page.screenshot({ path: "perf-results/setlist-editor.png" });

  await page.getByRole("button", { name: "Save setlist" }).click();
  expect(await dispatched(page)).toEqual([
    { SaveSetlist: { id: "sat", name: "Saturday late", entries: [{ id: 2, song_id: "s2" }, { id: 1, song_id: "s1" }, { id: 4, song_id: "s3" }], expected_revision: 2 } },
  ]);

  await page.getByRole("button", { name: "Play this setlist" }).click();
  expect((await dispatched(page))[1]).toEqual({ SetActiveSetlist: { id: "sat" } });
});

test("an empty name cannot be saved", async ({ page }) => {
  await page.addInitScript(tauriStandIn, view);
  await page.goto("/");
  await page.getByRole("navigation").getByRole("button", { name: "Setlists" }).click();
  await page.getByRole("button", { name: "New setlist" }).click();
  await page.getByLabel("Name").fill("  ");
  await expect(page.getByRole("button", { name: "Save setlist" })).toBeDisabled();
  await expect(page.getByText("Give the setlist a name.")).toBeVisible();
});

test("deleting a setlist asks first and then sends the revision that was shown", async ({ page }) => {
  await page.addInitScript(tauriStandIn, view);
  await page.goto("/");
  await page.getByRole("navigation").getByRole("button", { name: "Setlists" }).click();
  await page.getByRole("button", { name: /Sunday/ }).click();

  await page.getByRole("button", { name: "Delete setlist" }).click();
  await expect(page.getByRole("dialog", { name: "Delete Sunday?" })).toBeVisible();
  await expect(page.getByText("timeline order again")).toBeVisible();
  await page.screenshot({ path: "perf-results/setlist-delete.png" });

  await page.getByRole("button", { name: "Keep it" }).click();
  expect(await dispatched(page)).toEqual([]);

  await page.getByRole("button", { name: "Delete setlist" }).click();
  await page.getByRole("button", { name: "Delete", exact: true }).click();
  expect(await dispatched(page)).toEqual([{ DeleteSetlist: { id: "sun", expected_revision: 1 } }]);
});
