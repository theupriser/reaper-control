import { expect, test, type Page } from "@playwright/test";
import { tauriStandIn } from "./tauri-stand-in";

const catalog = { revision: 1, setlist_revision: 1, project_id: "project", songs: [], project_songs: [], cues: [], setlists: [], active_setlist: null };
const connected = { status: { Connected: { extension_version: "test" } }, live: null, catalog };
const down = { status: "NotRunning", live: null, catalog };

type Step = [string, string, string];
const installation = (steps: Step[], canInstall: boolean, complete = false) => ({
  steps: steps.map(([id, status, advice]) => ({ id, status, advice })),
  folder: "/Users/test/Library/Application Support/REAPER",
  can_install: canInstall,
  complete,
});
const fresh = installation([["FindReaper", "Done", ""], ["InstallExtension", "Current", "not installed"], ["RestartReaper", "Waiting", ""], ["Connect", "Waiting", ""]], true);
const installed = installation([["FindReaper", "Done", ""], ["InstallExtension", "Done", ""], ["RestartReaper", "Current", "Open REAPER. It loads the extension when it starts."], ["Connect", "Waiting", ""]], false);
const finished = installation([["FindReaper", "Done", ""], ["InstallExtension", "Done", ""], ["RestartReaper", "Done", ""], ["Connect", "Done", ""]], false, true);

const answer = (page: Page, command: string, value: unknown) => page.evaluate(([name, payload]) => (window as any).__answer(name, payload), [command, value] as const);

test("a first run walks from install to connected", async ({ page }) => {
  await page.addInitScript(tauriStandIn, down);
  await page.addInitScript((answers) => {
    const w = window as any;
    const wait = setInterval(() => { if (w.__answer) { clearInterval(wait); w.__answer("current_installation", answers.fresh); w.__answer("install_extension", answers.installed); } }, 1);
  }, { fresh, installed });
  await page.goto("/");

  await expect(page.getByRole("heading", { name: "Set up the connection" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Install the extension" })).toBeEnabled();
  await page.screenshot({ path: "perf-results/wizard-start.png" });

  await page.getByRole("button", { name: "Install the extension" }).click();
  await expect(page.getByText("Open REAPER. It loads the extension when it starts.")).toBeVisible();
  await expect(page.getByRole("button", { name: "Install the extension" })).toBeDisabled();

  await answer(page, "current_installation", finished);
  await expect(page.getByRole("status")).toContainText("Connected. You are ready.");
  await page.screenshot({ path: "perf-results/wizard-done.png" });
  await page.getByRole("button", { name: "Continue" }).click();
  await expect(page.getByRole("heading", { name: "Set up the connection" })).toHaveCount(0);
});

test("a person can skip the wizard and open it again from Settings", async ({ page }) => {
  await page.addInitScript(tauriStandIn, connected);
  await page.addInitScript((answers) => {
    const w = window as any;
    const wait = setInterval(() => { if (w.__answer) { clearInterval(wait); w.__answer("current_installation", answers.fresh); } }, 1);
  }, { fresh });
  await page.goto("/");
  await page.getByRole("button", { name: "Skip for now" }).click();
  await expect(page.getByRole("heading", { name: "Set up the connection" })).toHaveCount(0);

  await page.getByRole("navigation").getByRole("button", { name: "Settings" }).click();
  await page.getByRole("button", { name: "Open the setup" }).click();
  await expect(page.getByRole("heading", { name: "Set up the connection" })).toBeVisible();
});

test("an installation that is already in place does not open the wizard", async ({ page }) => {
  await page.addInitScript(tauriStandIn, connected);
  await page.goto("/");
  await page.getByRole("navigation").waitFor();
  await expect(page.getByRole("heading", { name: "Set up the connection" })).toHaveCount(0);
});

test("a connected link with no extension in REAPER's folder still offers to install it", async ({ page }) => {
  const connectedButMissing = installation([["FindReaper", "Done", ""], ["InstallExtension", "Current", "not installed"], ["RestartReaper", "Done", ""], ["Connect", "Done", ""]], true, true);
  await page.addInitScript(tauriStandIn, connected);
  await page.addInitScript((answer) => {
    const w = window as any;
    const wait = setInterval(() => { if (w.__answer) { clearInterval(wait); w.__answer("current_installation", answer); } }, 1);
  }, connectedButMissing);
  await page.goto("/");
  await page.getByRole("navigation").getByRole("button", { name: "Settings" }).click();
  await page.getByRole("button", { name: "Open the setup" }).click();

  await expect(page.getByRole("button", { name: "Install the extension" })).toBeEnabled();
  await expect(page.getByRole("button", { name: "Continue" })).toBeVisible();
});
