import { expect, test } from "@playwright/test";
import { tauriStandIn } from "./tauri-stand-in";

const catalog = { revision: 1, setlist_revision: 1, project_id: "project", songs: [], project_songs: [], cues: [], setlists: [], active_setlist: null };
const connected = { status: { Connected: { extension_version: "test" } }, live: null, catalog };

type Row = [string, string, string];
const checklist = (rows: Row[]) => ({
  items: rows.map(([id, status, detail]) => ({ id, status, detail })),
  ready: rows.every(([, status]) => status !== "Failed"),
});
const ok = checklist([
  ["ExtensionCurrent", "Passed", "The installed extension is the one that ships with this app."],
  ["Connected", "Passed", "Connected to extension test."],
  ["SongsFound", "Passed", "12 songs found."],
  ["SetlistValid", "Passed", "Setlist \"Friday\" is in order."],
  ["MidiPresent", "Skipped", "MIDI input is off in Settings."],
]);
const broken = checklist([
  ["ExtensionCurrent", "Failed", "Install the extension from Settings."],
  ["Connected", "Passed", "Connected to extension test."],
  ["SongsFound", "Passed", "12 songs found."],
  ["SetlistValid", "Failed", "Setlist \"Friday\" has 2 entries whose song is gone from the project. Fix them on Setlists."],
  ["MidiPresent", "Failed", "No MIDI device found. Plug in your controller."],
]);

const open = async (page: import("@playwright/test").Page, answer: unknown) => {
  await page.addInitScript(tauriStandIn, connected);
  await page.addInitScript((value) => {
    const w = window as any;
    const wait = setInterval(() => { if (w.__answer) { clearInterval(wait); w.__answer("current_checklist", value); } }, 1);
  }, answer);
  await page.goto("/");
  await page.getByRole("button", { name: "Pre-show check" }).click();
};

test("everything in order says ready", async ({ page }) => {
  await open(page, ok);
  await expect(page.getByRole("heading", { name: "Pre-show check" })).toBeVisible();
  await expect(page.getByText("12 songs found.")).toBeVisible();
  await expect(page.getByRole("status")).toContainText("Ready for the show.");
  await page.screenshot({ path: "perf-results/checklist-ready.png" });
});

test("failing rows are named and the extension row offers the setup", async ({ page }) => {
  await open(page, broken);
  await expect(page.getByRole("status")).toContainText("Not ready");
  await expect(page.getByText("No MIDI device found. Plug in your controller.")).toBeVisible();
  await expect(page.getByText("Fix this", { exact: true })).toHaveCount(3);
  await page.screenshot({ path: "perf-results/checklist-broken.png" });
  await page.getByRole("button", { name: "Open the setup" }).click();
  await expect(page.getByRole("heading", { name: "Set up the connection" })).toBeVisible();
});
