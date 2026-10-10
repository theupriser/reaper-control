import { describe, expect, it } from "vitest";
import { statRows } from "./health";

describe("statRows", () => {
  const stats = { machine_cpu_percent: 12.4, memory_used_megabytes: 4000, memory_total_megabytes: 16000, app_memory_megabytes: 90, reaper_memory_megabytes: null };

  it("rounds the numbers and says when REAPER was not found", () => {
    const rows = statRows(stats);
    expect(rows[0].value).toBe("12%");
    expect(rows[1].value).toBe("4000 MB / 16000 MB");
    expect(rows[3].value).toBe("not found");
  });
});
