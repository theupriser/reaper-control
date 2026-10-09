import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { InstallationView, WizardStep, WizardStepStatus } from "./generated/protocol";
import { needsWizard, watchInstallation } from "./wizard";

const step = (id: WizardStep["id"], status: WizardStepStatus): WizardStep => ({ id, status, advice: "" });
const view = (statuses: WizardStepStatus[], complete = false): InstallationView => ({
  steps: [step("FindReaper", statuses[0]), step("InstallExtension", statuses[1]), step("RestartReaper", statuses[2]), step("Connect", statuses[3])],
  folder: "/reaper",
  can_install: false,
  complete,
});

describe("needsWizard", () => {
  it("opens while the extension is not installed", () => {
    expect(needsWizard(view(["Done", "Current", "Waiting", "Waiting"]))).toBe(true);
    expect(needsWizard(view(["NeedsYou", "Waiting", "Waiting", "Waiting"]))).toBe(true);
  });

  it("stays shut once the extension is in place or connected", () => {
    expect(needsWizard(view(["Done", "Done", "Current", "Waiting"]))).toBe(false);
    expect(needsWizard(view(["Done", "Done", "Done", "Done"], true))).toBe(false);
  });
});

describe("watchInstallation", () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  it("looks at once and then on every interval until stopped", async () => {
    const read = vi.fn(async () => view(["Done", "Done", "Done", "Done"], true));
    const seen: InstallationView[] = [];
    const stop = watchInstallation(read, (next) => seen.push(next), 1000);
    await vi.advanceTimersByTimeAsync(2500);
    expect(read).toHaveBeenCalledTimes(3);
    stop();
    await vi.advanceTimersByTimeAsync(5000);
    expect(read).toHaveBeenCalledTimes(3);
    expect(seen).toHaveLength(3);
  });

  it("keeps going after a failed look", async () => {
    const read = vi.fn().mockRejectedValueOnce(new Error("no")).mockResolvedValue(view(["Done", "Done", "Done", "Done"], true));
    const seen: InstallationView[] = [];
    const stop = watchInstallation(read, (next) => seen.push(next), 1000);
    await vi.advanceTimersByTimeAsync(1100);
    stop();
    expect(seen).toHaveLength(1);
  });
});
