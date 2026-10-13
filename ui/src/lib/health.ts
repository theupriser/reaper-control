import type { SystemStats } from "./generated/protocol";
import { strings } from "./strings";

/** `percent` fills the bar of the row (0 to 100), or is null when there is nothing to show. */
export type StatRow = { label: string; value: string; percent: number | null };

const megabytes = (value: number) => strings.health.megabytes(Math.round(value));
const share = (part: number, whole: number): number => (whole > 0 ? Math.min(Math.max((part / whole) * 100, 0), 100) : 0);

/** The lines of the health dialog: machine use, then the two processes (as a share of the machine memory). */
export function statRows(stats: SystemStats): StatRow[] {
  return [
    { label: strings.health.machineCpu, value: `${Math.round(stats.machine_cpu_percent)}%`, percent: share(stats.machine_cpu_percent, 100) },
    { label: strings.health.machineMemory, value: `${megabytes(stats.memory_used_megabytes)} / ${megabytes(stats.memory_total_megabytes)}`, percent: share(stats.memory_used_megabytes, stats.memory_total_megabytes) },
    { label: strings.health.appMemory, value: megabytes(stats.app_memory_megabytes), percent: share(stats.app_memory_megabytes, stats.memory_total_megabytes) },
    { label: strings.health.reaperMemory, value: stats.reaper_memory_megabytes === null ? strings.health.notFound : megabytes(stats.reaper_memory_megabytes), percent: stats.reaper_memory_megabytes === null ? null : share(stats.reaper_memory_megabytes, stats.memory_total_megabytes) },
  ];
}
