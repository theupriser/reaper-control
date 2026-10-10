import type { LinkProblem, LinkStatus } from "./generated/protocol";
import { strings } from "./strings";

export type ConnectionBadge = { label: string; detail: string; tone: "ok" | "error" };

export function connectionBadge(status: LinkStatus, problem: LinkProblem | null = null): ConnectionBadge {
  if (problem) {
    const detail = problem.extension_outdated ? strings.connection.updateExtension : strings.connection.seeMessage;
    return { label: problem.message, detail, tone: "error" };
  }
  if (status === "NotRunning") {
    return { label: strings.connection.notRunning, detail: strings.connection.waitingForExtension, tone: "error" };
  }
  return { label: strings.connection.connected, detail: strings.connection.extensionVersion(status.Connected.extension_version), tone: "ok" };
}
