import type { InstallationView } from "./generated/protocol";

/** Whether the wizard should open by itself: the extension is not in place yet. */
export const needsWizard = (view: InstallationView): boolean =>
  !view.complete && view.steps.slice(0, 2).some((step) => step.status !== "Done");

/** Reads the installation now and then every `intervalMilliseconds`; returns the function that stops it. */
export function watchInstallation(
  read: () => Promise<InstallationView>,
  onView: (view: InstallationView) => void,
  intervalMilliseconds: number,
): () => void {
  let stopped = false;
  const look = async () => {
    try {
      const view = await read();
      if (!stopped) onView(view);
    } catch {
      // the next look tries again
    }
  };
  void look();
  const timer = setInterval(look, intervalMilliseconds);
  return () => {
    stopped = true;
    clearInterval(timer);
  };
}
