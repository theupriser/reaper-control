import { describe, expect, it } from "vitest";
import { healthBanner, statRows } from "./health";

describe("healthBanner", () => {
  it("is empty while the extension is connected", () => {
    expect(healthBanner({ Connected: { extension_version: "1" } }, null)).toBeNull();
  });

  it("names a problem the app found", () => {
    expect(healthBanner({ Connected: { extension_version: "1" } }, "The extension is not loaded")?.title).toBe("The extension is not loaded");
  });

  it("asks to open REAPER when nothing is running", () => {
    expect(healthBanner("NotRunning", null)).toEqual({ title: "REAPER not running", text: "Open REAPER. We keep trying." });
  });
});

describe("statRows", () => {
  const stats = { machine_cpu_percent: 12.4, memory_used_megabytes: 4000, memory_total_megabytes: 16000, app_memory_megabytes: 90, reaper_memory_megabytes: null };

  it("rounds the numbers and says when REAPER was not found", () => {
    const rows = statRows(stats);
    expect(rows[0].value).toBe("12%");
    expect(rows[1].value).toBe("4000 MB / 16000 MB");
    expect(rows[3].value).toBe("not found");
  });
});
