import { describe, expect, it } from "vitest";
import { budgetFailures, frameBudget, percentile, summary } from "./frame-budget";

describe("frame budget", () => {
  it("takes percentiles by nearest rank", () => {
    const samples = Array.from({ length: 100 }, (_, n) => n + 1);
    expect(percentile(samples, 0.5)).toBe(50);
    expect(percentile(samples, 0.95)).toBe(95);
    expect(percentile(samples, 1)).toBe(100);
    expect(percentile([], 0.95)).toBe(0);
  });

  it("passes samples inside the budget", () => {
    expect(budgetFailures([8, 12, 16, 20, 30], frameBudget)).toEqual([]);
  });

  it("fails when the p95 or the maximum is over", () => {
    expect(budgetFailures([60, 70, 80, 90, 95], frameBudget)[0]).toContain("p95");
    const spike = [...Array.from({ length: 99 }, () => 10), 150];
    expect(budgetFailures(spike, frameBudget)).toHaveLength(1);
    expect(budgetFailures(spike, frameBudget)[0]).toContain("maximum");
  });

  it("fails when nothing was measured", () => {
    expect(budgetFailures([], frameBudget)).toEqual(["no samples were measured"]);
  });

  it("prints one line", () => {
    expect(summary("click", [10, 20])).toContain("n=  2");
  });
});
