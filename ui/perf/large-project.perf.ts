import { expect, test, type Page } from "@playwright/test";
import { budgetFailures, frameBudget, summary } from "../src/lib/frame-budget";
import { inputToPaintRecorder, tauriStandIn } from "./tauri-stand-in";

const songCount = Number(process.env.PERF_SONGS ?? 300);

const song = (number: number) => ({ id: `s${number}`, number, name: `Song ${number}`, start: number * 200, end: number * 200 + 200, colour: null, hard_stop: number % 7 === 0, length: null, bpm: 120 });
const live = (sequence: number, position: number, current: number) => ({
  sequence, timestamp: sequence, transport: "Playing", position, phase: "Playing", setlist_id: "set", current_song: current, next_song: current + 1,
  autoplay: true, count_in: false, record_armed: false, catalog_revision: 1, setlist_revision: 1,
});
const songs = Array.from({ length: songCount }, (_, at) => song(at + 1));
const view = (sequence: number, position: number, current: number) => ({
  status: { Connected: { extension_version: "test" } },
  live: live(sequence, position, current),
  catalog: {
    revision: 1, setlist_revision: 1, project_id: "project", songs, project_songs: songs, cues: [],
    setlists: [{ id: "set", name: "Set", revision: 1, entries: songs.map((each, at) => ({ id: at + 1, song_id: each.id })) }],
    active_setlist: "set",
  },
});

const samples = (page: Page) => page.evaluate(() => (window as any).__samples as { label: string; milliseconds: number }[]);

test(`a project of ${songCount} songs stays inside the budget`, async ({ page }) => {
  await page.addInitScript(tauriStandIn, view(1, 10, 0));
  await page.addInitScript(inputToPaintRecorder, 0);
  await page.goto("/");
  await page.getByRole("navigation").waitFor();

  for (const screen of ["Player", "Setlists"]) {
    await page.getByRole("navigation").getByRole("button", { name: screen }).click();
    if (screen === "Setlists") await page.getByText("Set", { exact: true }).first().click();
    await page.waitForTimeout(200);
    for (let push = 0; push < 30; push++) {
      await page.evaluate(([pushed, label]) => {
        const start = performance.now();
        (window as any).__pushEvent("link-view", pushed);
        (window as any).__mark(label, start);
      }, [view(push + 2, 10 + push, Math.floor(push / 15)), `${screen} push`] as const);
      await page.waitForTimeout(40);
    }
  }
  const rows = await page.getByRole("list", { name: "Songs in this setlist" }).getByRole("listitem").count();
  console.log(`list items on the setlists screen: ${rows}`);

  const all = await samples(page);
  const failures: string[] = [];
  for (const label of ["Player push", "Setlists push"]) {
    const milliseconds = all.filter((sample) => sample.label === label).map((sample) => sample.milliseconds);
    console.log(summary(`${label} (${songCount} songs)`, milliseconds));
    failures.push(...budgetFailures(milliseconds, frameBudget).map((failure) => `${label}: ${failure}`));
  }
  expect(failures, failures.join("; ")).toEqual([]);
});
