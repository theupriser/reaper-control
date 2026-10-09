import { expect, test, type Page } from "@playwright/test";
import { budgetFailures, frameBudget, summary } from "../src/lib/frame-budget";
import { inputToPaintRecorder, tauriStandIn } from "./tauri-stand-in";

const slowDown = Number(process.env.PERF_SLOW_MILLISECONDS ?? 0);

const song = (number: number, start: number) => ({ id: `s${number}`, number, name: `Song ${number}`, start, end: start + 200, colour: null, hard_stop: false, length: null, bpm: 120 });
const live = (sequence: number, position: number) => ({
  sequence, timestamp: sequence, transport: "Playing", position, phase: "Playing", setlist_id: "set", current_song: 0, next_song: 1,
  autoplay: true, count_in: false, record_armed: false, catalog_revision: 1, setlist_revision: 1,
});
const view = {
  status: { Connected: { extension_version: "test" } },
  live: live(1, 10),
  catalog: {
    revision: 1, setlist_revision: 1, project_id: "project", songs: [song(1, 0), song(2, 200), song(3, 400)], project_songs: [], cues: [],
    setlists: [{ id: "set", name: "Set", revision: 1, entries: [{ id: 1, song_id: "s1" }, { id: 2, song_id: "s2" }, { id: 3, song_id: "s3" }] }],
    active_setlist: "set",
  },
};

const samples = (page: Page) => page.evaluate(() => (window as any).__samples as { label: string; milliseconds: number }[]);

test("input to paint stays inside the budget", async ({ page }) => {
  await page.addInitScript(tauriStandIn, view);
  await page.addInitScript(inputToPaintRecorder, slowDown);
  await page.goto("/");
  await page.getByRole("navigation").waitFor();

  for (let round = 0; round < 6; round++) {
    for (const name of ["Settings", "Help", "Setlists", "Player"]) {
      await page.getByRole("navigation").getByRole("button", { name }).click();
      await page.waitForTimeout(60);
    }
    await page.getByRole("button", { name: "Performer mode" }).click();
    await page.waitForTimeout(60);
    for (const key of [" ", "ArrowRight", "ArrowLeft", " "]) {
      await page.keyboard.press(key);
      await page.waitForTimeout(60);
    }
    await page.keyboard.press("Escape");
    await page.waitForTimeout(60);
    for (let push = 0; push < 5; push++) {
      await page.evaluate(([pushed]) => {
        const start = performance.now();
        (window as any).__pushEvent("link-view", pushed);
        (window as any).__mark("state push", start);
      }, [{ ...view, live: live(round * 10 + push + 2, 10 + round * 5 + push) }]);
      await page.waitForTimeout(40);
    }
  }

  const all = await samples(page);
  const inputs = all.filter((sample) => sample.label !== "state push").map((sample) => sample.milliseconds);
  const pushes = all.filter((sample) => sample.label === "state push").map((sample) => sample.milliseconds);
  console.log(summary("key and click", inputs));
  console.log(summary("state push", pushes));
  const failures = [...budgetFailures(inputs, frameBudget).map((f) => `input: ${f}`), ...budgetFailures(pushes, frameBudget).map((f) => `push: ${f}`)];
  expect(failures, failures.join("; ")).toEqual([]);
});
