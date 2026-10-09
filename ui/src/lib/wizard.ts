import type { InstallationView } from "./generated/protocol";
import { watchView } from "./watch";

/** Whether the wizard should open by itself: the extension is not in place yet. */
export const needsWizard = (view: InstallationView): boolean =>
  !view.complete && view.steps.slice(0, 2).some((step) => step.status !== "Done");

/** Reads the installation now and then every `intervalMilliseconds`; returns the function that stops it. */
export const watchInstallation = watchView<InstallationView>;
