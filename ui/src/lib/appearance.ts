export const themes = ["dark", "stage-dark", "light"] as const;
export type Theme = (typeof themes)[number];
export const densities = ["comfortable", "compact"] as const;
export type Density = (typeof densities)[number];
export const touchSizes = ["normal", "large"] as const;
export type Touch = (typeof touchSizes)[number];

export interface Appearance {
  theme: Theme;
  density: Density;
  touch: Touch;
}

/** The look the app saved; a value this build does not know falls back to the default. */
export function appearanceOf(saved: { theme: string; density: string; touch: string }): Appearance {
  const pick = <T extends string>(value: string, allowed: readonly T[], fallback: T): T => (allowed as readonly string[]).includes(value) ? (value as T) : fallback;
  return {
    theme: pick(saved.theme, themes, defaultAppearance.theme),
    density: pick(saved.density, densities, defaultAppearance.density),
    touch: pick(saved.touch, touchSizes, defaultAppearance.touch),
  };
}

export const defaultAppearance: Appearance = { theme: "dark", density: "comfortable", touch: "normal" };

/** Sets the data attributes the token sheet reads. */
export function applyAppearance(root: { dataset: Record<string, string | undefined> }, appearance: Appearance): void {
  root.dataset.theme = appearance.theme;
  root.dataset.density = appearance.density;
  root.dataset.touch = appearance.touch;
}
