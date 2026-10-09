export const themes = ["dark", "stage-dark", "light"] as const;
export type Theme = (typeof themes)[number];
export type Density = "comfortable" | "compact";
export type Touch = "normal" | "large";

export interface Appearance {
  theme: Theme;
  density: Density;
  touch: Touch;
}

export const defaultAppearance: Appearance = { theme: "dark", density: "comfortable", touch: "normal" };

/** Sets the data attributes the token sheet reads. */
export function applyAppearance(root: { dataset: Record<string, string | undefined> }, appearance: Appearance): void {
  root.dataset.theme = appearance.theme;
  root.dataset.density = appearance.density;
  root.dataset.touch = appearance.touch;
}
