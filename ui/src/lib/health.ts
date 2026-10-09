import type { LinkStatus, SystemStats } from "./generated/protocol";
import { strings } from "./strings";

export type HealthBanner = { title: string; text: string };

/** The banner for a link that does not work; none while it does. */
export function healthBanner(status: LinkStatus, problem: string | null): HealthBanner | null {
  if (problem) return { title: problem, text: strings.health.keepTrying };
  if (status === "NotRunning") return { title: strings.connection.notRunning, text: strings.health.openReaper };
  return null;
}

export type StatRow = { label: string; value: string };

const megabytes = (value: number) => strings.health.megabytes(Math.round(value));

/** The lines of the health dialog: machine use, then the two processes. */
export function statRows(stats: SystemStats): StatRow[] {
  return [
    { label: strings.health.machineCpu, value: `${Math.round(stats.machine_cpu_percent)}%` },
    { label: strings.health.machineMemory, value: `${megabytes(stats.memory_used_megabytes)} / ${megabytes(stats.memory_total_megabytes)}` },
    { label: strings.health.appMemory, value: megabytes(stats.app_memory_megabytes) },
    { label: strings.health.reaperMemory, value: stats.reaper_memory_megabytes === null ? strings.health.notFound : megabytes(stats.reaper_memory_megabytes) },
  ];
}
