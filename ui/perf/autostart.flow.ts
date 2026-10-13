import { expect, test, type Page } from "@playwright/test";
import { tauriStandIn } from "./tauri-stand-in";

const view = (autostart: boolean, connected = true) => ({
  status: connected ? { Connected: { extension_version: "1.0.7" } } : "NotRunning",
  live: null,
  catalog: { revision: 1, setlist_revision: 1, project_id: "project", songs: [], project_songs: [], cues: [], setlists: [], active_setlist: null, autostart },
});
const dispatched = (page: Page) => page.evaluate(() => (window as any).__dispatched ?? []);
const sidebarSwitch = (page: Page) => page.getByRole("navigation").getByRole("switch", { name: "Start with this project" });

test("the sidebar switch asks the extension to start the app with this project", async ({ page }) => {
  await page.addInitScript(tauriStandIn, view(false));
  await page.goto("/");
  await expect(sidebarSwitch(page)).toHaveAttribute("aria-checked", "false");

  await sidebarSwitch(page).click();
  expect(await dispatched(page)).toEqual([{ SetAutostart: { enabled: true } }]);
});

test("the switch shows what the project says, in the sidebar and in Settings", async ({ page }) => {
  await page.addInitScript(tauriStandIn, view(true));
  await page.goto("/");
  await expect(sidebarSwitch(page)).toHaveAttribute("aria-checked", "true");

  await page.getByRole("navigation").getByRole("button", { name: "Settings" }).click();
  const inSettings = page.getByRole("main").getByRole("switch", { name: "Start with this project" });
  await expect(inSettings).toHaveAttribute("aria-checked", "true");
  await inSettings.click();
  expect(await dispatched(page)).toEqual([{ SetAutostart: { enabled: false } }]);
});

test("without REAPER the switch cannot be used", async ({ page }) => {
  await page.addInitScript(tauriStandIn, view(true, false));
  await page.goto("/");
  await expect(sidebarSwitch(page)).toBeDisabled();
  await expect(page.getByText("Connect to REAPER to change this.").first()).toBeVisible();
});
