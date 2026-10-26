import type { InstallationView, Phase } from "./generated/protocol";

/** Phases in which a show is on; nothing may close REAPER then. */
const SHOW_ON: Phase[] = ["Playing", "Paused", "CountingIn", "HardStopped", "HandingOver"];

/** Whether the person is offered the extension update now: one is ready and no show is running. */
export const updateOffered = (view: InstallationView | null, phase: Phase | undefined): boolean =>
  view !== null && view.update_available && !(phase !== undefined && SHOW_ON.includes(phase));
