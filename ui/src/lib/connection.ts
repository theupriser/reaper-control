import type { LinkStatus } from "./generated/protocol";

export type ConnectionBadge = { label: string; detail: string; tone: "ok" | "warn" };

export function connectionBadge(status: LinkStatus): ConnectionBadge {
  if (status === "NotRunning") {
    return { label: "REAPER not running", detail: "waiting for the extension", tone: "warn" };
  }
  return { label: "Connected", detail: `extension ${status.Connected.extension_version}`, tone: "ok" };
}
