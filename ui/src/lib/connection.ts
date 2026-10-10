import type { LinkProblem, LinkStatus } from "./generated/protocol";
import { strings } from "./strings";

/** `line` is the short text of the sidebar; `label` and `detail` give the reason, in the health dialog. */
export type ConnectionBadge = { label: string; detail: string; tone: "ok" | "error"; line?: string };

export function connectionBadge(status: LinkStatus, problem: LinkProblem | null = null, bundledVersion: string | null = null): ConnectionBadge {
  if (problem) {
    const detail = problem.extension_outdated ? strings.connection.updateExtension : strings.connection.seeMessage;
    return { label: problem.message, detail, tone: "error", line: problem.extension_outdated ? strings.connection.outdatedLine : strings.connection.disconnectedLine };
  }
  if (status === "NotRunning") {
    return { label: strings.connection.notRunning, detail: strings.connection.waitingForExtension, tone: "error", line: strings.connection.disconnectedLine };
  }
  if (bundledVersion !== null && status.Connected.extension_version !== bundledVersion) {
    return { label: strings.connection.outdated, detail: `${strings.connection.extensionVersion(status.Connected.extension_version)}, ${strings.connection.updateExtension}`, tone: "error", line: strings.connection.outdatedLine };
  }
  return { label: strings.connection.connected, detail: strings.connection.extensionVersion(status.Connected.extension_version), tone: "ok", line: strings.connection.connectedLine };
}
