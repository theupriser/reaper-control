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

const openSetlists = async (page: import("@playwright/test").Page) => {
  await page.addInitScript(tauriStandIn, view);
  await page.goto("/");
  await page.getByRole("navigation").getByRole("button", { name: "Setlists" }).click();
};

const dispatched = (page: import("@playwright/test").Page) => page.evaluate(() => (window as any).__dispatched ?? []);

test("the setlists screen shows a card for each setlist", async ({ page }) => {
  await openSetlists(page);
  const saturday = page.getByRole("region", { name: "Saturday" });
  await expect(saturday).toContainText("3 songs");
  await expect(saturday).toContainText("Opener, Ballad, gone");
  await expect(saturday).toContainText("1 song is no longer in the project");
  const sunday = page.getByRole("region", { name: "Sunday" });
  await expect(sunday).toContainText("Playing");
  await expect(sunday).not.toContainText("no longer in the project");
  await page.screenshot({ path: "perf-results/setlist-cards.png" });
});

test("edit a setlist, save it and play it", async ({ page }) => {
  await openSetlists(page);
  await page.getByRole("region", { name: "Saturday" }).getByRole("button", { name: "Edit songs" }).click();
  await expect(page.getByText("Not in the project")).toBeVisible();
  await expect(page.getByText("no longer in the REAPER project")).toBeVisible();
  await expect(page.getByRole("button", { name: "Save setlist" })).toBeDisabled();

  await page.getByRole("button", { name: "Remove Opener" }).click();
  await page.getByRole("button", { name: "Remove gone" }).click();
  await page.getByRole("button", { name: "Add Closer" }).click();
  await page.getByLabel("Name").fill("Saturday late");
  await expect(page.getByRole("button", { name: "Play this setlist" })).toBeDisabled();
  await page.screenshot({ path: "perf-results/setlist-editor.png" });

  await page.getByRole("button", { name: "Save setlist" }).click();
  expect(await dispatched(page)).toEqual([
    { SaveSetlist: { id: "sat", name: "Saturday late", entries: [{ id: 2, song_id: "s2" }, { id: 4, song_id: "s3" }], expected_revision: 2 } },
  ]);
});

test("the editor plays the setlist once it is saved", async ({ page }) => {
  await openSetlists(page);
  await page.getByRole("region", { name: "Saturday" }).getByRole("button", { name: "Edit songs" }).click();
  await page.getByRole("button", { name: "Play this setlist" }).click();
  expect(await dispatched(page)).toEqual([{ SetActiveSetlist: { id: "sat" } }]);
  await expect(page.getByRole("heading", { name: "Setlists" })).toHaveCount(0);
});

test("leaving some project songs out is not a problem", async ({ page }) => {
  await openSetlists(page);
  await page.getByRole("region", { name: "Sunday" }).getByRole("button", { name: "Edit songs" }).click();
  await expect(page.getByRole("button", { name: "All setlists" }).locator("xpath=../..")).toContainText("1 song · 1:30");
  await expect(page.getByText("In setlist")).toHaveCount(1);
  await expect(page.getByRole("status")).toHaveCount(0);
});

test("dragging reorders the setlist and adds a song from the project", async ({ page }) => {
  await openSetlists(page);
  await page.getByRole("region", { name: "Saturday" }).getByRole("button", { name: "Edit songs" }).click();
  await page.getByRole("button", { name: "Remove gone" }).click();

  const rowOf = (name: string) => page.getByRole("list", { name: "Songs in this setlist" }).getByRole("listitem").filter({ hasText: name });
  const centre = async (locator: import("@playwright/test").Locator) => {
    const box = await locator.boundingBox();
    if (!box) throw new Error("not on screen");
    return { x: box.x + box.width / 2, y: box.y + box.height / 2, top: box.y };
  };

  const ballad = await centre(page.getByRole("img", { name: "Drag Ballad to reorder" }));
  const opener = await centre(rowOf("Opener"));
  await page.mouse.move(ballad.x, ballad.y);
  await page.mouse.down();
  await page.mouse.move(ballad.x, opener.top + 4, { steps: 8 });
  await page.screenshot({ path: "perf-results/setlist-dragging.png" });
  await page.mouse.up();
  await expect(rowOf("Ballad").locator(".position")).toHaveText("1");

  const closer = await centre(page.getByRole("img", { name: "Drag Closer to the setlist" }));
  const first = await centre(rowOf("Ballad"));
  await page.mouse.move(closer.x, closer.y);
  await page.mouse.down();
  await page.mouse.move(first.x, first.top + 4, { steps: 8 });
  await page.mouse.up();
  await expect(rowOf("Closer").locator(".position")).toHaveText("1");

  await page.getByRole("button", { name: "Save setlist" }).click();
  expect(await dispatched(page)).toEqual([
    { SaveSetlist: { id: "sat", name: "Saturday", entries: [{ id: 4, song_id: "s3" }, { id: 2, song_id: "s2" }, { id: 1, song_id: "s1" }], expected_revision: 2 } },
  ]);
});

test("going back with unsaved changes asks first", async ({ page }) => {
  await openSetlists(page);
  await page.getByRole("region", { name: "Sunday" }).getByRole("button", { name: "Edit songs" }).click();
  await page.getByRole("button", { name: "All setlists" }).click();
  await expect(page.getByRole("region", { name: "Sunday" })).toBeVisible();

  await page.getByRole("region", { name: "Sunday" }).getByRole("button", { name: "Edit songs" }).click();
  await page.getByRole("button", { name: "Add Opener" }).click();
  await page.getByRole("button", { name: "All setlists" }).click();
  await expect(page.getByRole("dialog", { name: "Leave without saving?" })).toBeVisible();
  await page.getByRole("button", { name: "Keep editing" }).click();
  await expect(page.getByLabel("Name")).toHaveValue("Sunday");
  await page.getByRole("button", { name: "All setlists" }).click();
  await page.getByRole("button", { name: "Leave and discard" }).click();
  await expect(page.getByRole("region", { name: "Sunday" })).toContainText("1 song");
});

test("an empty name cannot be saved", async ({ page }) => {
  await openSetlists(page);
  await page.getByRole("button", { name: /New setlist/ }).click();
  await page.getByLabel("Name").fill("  ");
  await expect(page.getByRole("button", { name: "Save setlist" })).toBeDisabled();
  await expect(page.getByText("Give the setlist a name.")).toBeVisible();
});

test("deleting a setlist asks first and then sends the revision that was shown", async ({ page }) => {
  await openSetlists(page);
  await page.getByRole("button", { name: "Delete Sunday" }).click();
  await expect(page.getByRole("dialog", { name: "Delete Sunday?" })).toBeVisible();
  await expect(page.getByText("timeline order again")).toBeVisible();
  await page.screenshot({ path: "perf-results/setlist-delete.png" });

  await page.getByRole("button", { name: "Keep it" }).click();
  expect(await dispatched(page)).toEqual([]);

  await page.getByRole("button", { name: "Delete Sunday" }).click();
  await page.getByRole("button", { name: "Delete", exact: true }).click();
  expect(await dispatched(page)).toEqual([{ DeleteSetlist: { id: "sun", expected_revision: 1 } }]);
});

test("a card plays its setlist", async ({ page }) => {
  await openSetlists(page);
  await page.getByRole("region", { name: "Saturday" }).getByRole("button", { name: "Play this" }).click();
  expect(await dispatched(page)).toEqual([{ SetActiveSetlist: { id: "sat" } }]);
  await expect(page.getByRole("heading", { name: "Setlists" })).toHaveCount(0);
});
