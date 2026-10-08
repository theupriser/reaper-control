import type { LinkStatus } from "./generated/protocol";

export type ConnectionBadge = { label: string; detail: string; tone: "ok" | "error" };

export function connectionBadge(status: LinkStatus, problem: string | null = null): ConnectionBadge {
  if (problem) return { label: problem, detail: "see the message at the bottom", tone: "error" };
  if (status === "NotRunning") {
    return { label: "REAPER not running", detail: "waiting for the extension", tone: "error" };
  }
  return { label: "Connected", detail: `extension ${status.Connected.extension_version}`, tone: "ok" };
}
