import { describe, expect, it } from "vitest";
import { applyAppearance, defaultAppearance, themes } from "./appearance";

import rawCss from "../tokens.css?raw";

const css = rawCss.replace(/\r\n/g, "\n");

const sources: Record<string, string> = import.meta.glob(["../components/*.svelte", "../kit/*.svelte"], {
  query: "?raw",
  import: "default",
  eager: true,
});

function block(selector: string): string {
  const start = css.indexOf(selector + " {");
  expect(start, selector).toBeGreaterThanOrEqual(0);
  return css.slice(start, css.indexOf("}", start));
}

function names(text: string): string[] {
  return [...text.matchAll(/(--[a-z0-9-]+):/g)].map((match) => match[1]);
}

describe("tokens", () => {
  const colours = names(block(':root,\n:root[data-theme="dark"]'));

  it("lets stage-dark and light override only tokens that dark defines", () => {
    for (const theme of themes.filter((name) => name !== "dark")) {
      for (const name of names(block(`:root[data-theme="${theme}"]`))) {
        expect(colours, `${theme} ${name}`).toContain(name);
      }
    }
  });

  it("gives the light theme its own text, background and accents", () => {
    const light = names(block(':root[data-theme="light"]'));
    for (const name of ["--bg", "--panel", "--text", "--muted", "--green", "--red", "--on-accent"]) {
      expect(light).toContain(name);
    }
  });

  it("keeps raw colours out of components", () => {
    for (const [file, source] of Object.entries(sources)) {
      expect(source, file).not.toMatch(/#[0-9a-fA-F]{3,8}\b|rgba?\(|(?<![-\w])(white|black)(?![-\w])/);
    }
  });

  it("applies the appearance as data attributes", () => {
    const root = { dataset: {} as Record<string, string | undefined> };
    applyAppearance(root, { ...defaultAppearance, theme: "stage-dark", touch: "large" });
    expect(root.dataset).toMatchObject({ theme: "stage-dark", density: "comfortable", touch: "large" });
  });
});

describe("motion", () => {
  it("takes every animated duration from a motion token, so reduced motion switches them off", () => {
    for (const [file, source] of Object.entries(sources)) {
      for (const [, value] of source.matchAll(/\btrans[i]tion:\s*([^;]+);/g)) {
        expect(value, file).not.toMatch(/\d(ms|s)\b/);
      }
    }
  });
});
