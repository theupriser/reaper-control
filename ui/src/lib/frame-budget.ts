/** How long the interface may take from an input (or a pushed state) to the next painted frame. */
export interface FrameBudget {
  p95Milliseconds: number;
  maximumMilliseconds: number;
}

export const frameBudget: FrameBudget = { p95Milliseconds: 50, maximumMilliseconds: 100 };

/** The value below which `fraction` (0 to 1) of the samples fall; nearest-rank. */
export function percentile(samples: number[], fraction: number): number {
  if (samples.length === 0) return 0;
  const sorted = [...samples].sort((a, b) => a - b);
  const rank = Math.ceil(fraction * sorted.length);
  return sorted[Math.min(sorted.length, Math.max(1, rank)) - 1];
}

/** What the samples got wrong against the budget; empty when they are within it. */
export function budgetFailures(samples: number[], budget: FrameBudget): string[] {
  if (samples.length === 0) return ["no samples were measured"];
  const failures: string[] = [];
  const p95 = percentile(samples, 0.95);
  const maximum = Math.max(...samples);
  if (p95 > budget.p95Milliseconds) failures.push(`p95 ${p95.toFixed(1)} ms is over ${budget.p95Milliseconds} ms`);
  if (maximum > budget.maximumMilliseconds) failures.push(`maximum ${maximum.toFixed(1)} ms is over ${budget.maximumMilliseconds} ms`);
  return failures;
}

/** One line for the log: count, median, p95 and maximum. */
export function summary(label: string, samples: number[]): string {
  const fixed = (value: number) => value.toFixed(1).padStart(6);
  return `${label.padEnd(18)} n=${String(samples.length).padStart(3)}  median${fixed(percentile(samples, 0.5))} ms  p95${fixed(percentile(samples, 0.95))} ms  max${fixed(Math.max(0, ...samples))} ms`;
}
