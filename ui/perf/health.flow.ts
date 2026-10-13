import { expect, test } from "@playwright/test";
import { tauriStandIn } from "./tauri-stand-in";

const connected = { Connected: { extension_version: "test" } };
const catalog = { revision: 1, setlist_revision: 1, project_id: "project", songs: [], project_songs: [], cues: [], setlists: [], active_setlist: null };
const view = (status: unknown) => ({ status, live: null, catalog });

test("a working link shows no alert", async ({ page }) => {
  await page.addInitScript(tauriStandIn, view(connected));
  await page.goto("/");
  await expect(page.getByRole("navigation").getByText("Connected")).toBeVisible();
  await expect(page.getByRole("alert")).toHaveCount(0);
});

test("a link that is down shows only the sidebar indicator, and the dialog shows the numbers", async ({ page }) => {
  await page.addInitScript(tauriStandIn, view("NotRunning"));
  await page.goto("/");
  await expect(page.getByRole("button", { name: "Connection details" })).toContainText("REAPER disconnected");
  await expect(page.getByRole("alert")).toHaveCount(0);

  await page.getByRole("button", { name: "Connection details" }).click();
  const dialog = page.getByRole("dialog", { name: "Connection and health" });
  await expect(dialog).toContainText("REAPER not running");
  await expect(dialog).toContainText("Machine CPU");
  await expect(dialog).toContainText("12%");
  await expect(dialog).toContainText("4000 MB / 16000 MB");
  await expect(dialog).toContainText("600 MB");
  await page.screenshot({ path: "perf-results/health.png" });
  await dialog.getByRole("button", { name: "Close" }).click();
  await expect(dialog).toBeHidden();

  const overflow = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
  expect(overflow).toBeLessThanOrEqual(0);
});

test("a problem the app found shows in the sidebar indicator and clears again", async ({ page }) => {
  await page.addInitScript(tauriStandIn, view(connected));
  await page.goto("/");
  await page.getByRole("navigation").waitFor();
  await page.evaluate(() => (window as any).__pushEvent("link-problem", { message: "The extension is not loaded", extension_outdated: false, extension_faulted: false }));
  const indicator = page.getByRole("button", { name: "Connection details" });
  await expect(indicator).toContainText("REAPER disconnected");
  await expect(page.getByRole("alert")).toHaveCount(0);
  await indicator.click();
  const dialog = page.getByRole("dialog", { name: "Connection and health" });
  await expect(dialog).toContainText("The extension is not loaded");
  await dialog.getByRole("button", { name: "Close" }).click();
  await page.evaluate(() => (window as any).__pushEvent("link-problem", null));
  await expect(indicator).toContainText("REAPER connected");
});

test("a link notice stays out of the toasts while other notices still show", async ({ page }) => {
  await page.addInitScript(tauriStandIn, view("NotRunning"));
  await page.goto("/");
  await page.getByRole("navigation").waitFor();
  await page.evaluate(() => (window as any).__pushEvent("notice", { key: "link", level: "Error", title: "REAPER is not running", text: "Open REAPER." }));
  await page.evaluate(() => (window as any).__pushEvent("notice", { key: "command", level: "Warning", title: "Refused", text: "Not now." }));
  await expect(page.getByText("Refused")).toBeVisible();
  await expect(page.getByText("Open REAPER.")).toHaveCount(0);
});

test("the numbers in the health dialog keep updating while it is open", async ({ page }) => {
  await page.addInitScript(tauriStandIn, view("NotRunning"));
  await page.goto("/");
  await page.getByRole("button", { name: "Connection details" }).click();
  const dialog = page.getByRole("dialog", { name: "Connection and health" });
  await expect(dialog).toContainText("12%");

  await page.evaluate(() =>
    (window as any).__answer("current_system_stats", { machine_cpu_percent: 57, memory_used_megabytes: 5000, memory_total_megabytes: 16000, app_memory_megabytes: 90, reaper_memory_megabytes: 700 }),
  );
  await expect(dialog).toContainText("57%");
  await expect(dialog).toContainText("700 MB");
  await expect(dialog.locator(".bar")).toHaveCount(4);
  await page.evaluate(() =>
    (window as any).__answer("current_system_stats", { machine_cpu_percent: 86, memory_used_megabytes: 9000, memory_total_megabytes: 16000, app_memory_megabytes: 240, reaper_memory_megabytes: 2400 }),
  );
  await expect(dialog.locator(".bar.high")).toHaveCount(1);
  await expect(dialog.locator(".bar.medium")).toHaveCount(1);
  await page.screenshot({ path: "perf-results/health-bars.png" });
});

test("a crashed extension can be switched back on from the health dialog", async ({ page }) => {
  await page.addInitScript(tauriStandIn, view("NotRunning"));
  await page.goto("/");
  await page.getByRole("navigation").waitFor();
  await page.evaluate(() => (window as any).__pushEvent("link-problem", { message: "The extension turned itself off", extension_outdated: false, extension_faulted: true }));
  await page.getByRole("button", { name: "Connection details" }).click();
  const dialog = page.getByRole("dialog", { name: "Connection and health" });
  await page.screenshot({ path: "perf-results/health-faulted.png" });
  await dialog.getByRole("button", { name: "Switch the extension back on" }).click();
  await expect(dialog).toContainText("Restart REAPER to use it");
  await expect(dialog.getByRole("button", { name: "Switch the extension back on" })).toHaveCount(0);
});
