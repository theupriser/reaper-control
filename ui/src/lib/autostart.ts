import type { LinkView } from "./generated/protocol";
import { strings } from "./strings";

/** What the two autostart switches show for the open project. */
export interface AutostartView {
  /** The project asks for the app to be started when it opens. */
  on: boolean;
  /** The switch can be used: there is a project to save the choice in. */
  available: boolean;
  /** Why the switch cannot do its work yet, or null. */
  problem: string | null;
}

/** `ready`: the extension knows where the app is installed. */
export function autostartView(link: LinkView, ready: boolean): AutostartView {
  const available = link.status !== "NotRunning";
  const on = available && link.catalog.autostart;
  return {
    on,
    available,
    problem: !available ? strings.autostart.needsReaper : on && !ready ? strings.autostart.unknownLocation : null,
  };
}
