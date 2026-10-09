import { describe, expect, it } from "vitest";
import { strings } from "./strings";

const sources: Record<string, string> = import.meta.glob("../components/*.svelte", {
  query: "?raw",
  import: "default",
  eager: true,
});
const components = Object.entries(sources);

/** The markup of a component: no script, no style, no comments, no `{expressions}`. */
const markup = (source: string): string => {
  let text = source
    .replace(/<script[\s\S]*?<\/script>/g, "")
    .replace(/<style[\s\S]*?<\/style>/g, "")
    .replace(/<!--[\s\S]*?-->/g, "");
  let previous = "";
  while (previous !== text) {
    previous = text;
    text = text.replace(/\{[^{}]*\}/g, "");
  }
  return text;
};

describe("strings", () => {
  it.each(components)("%s holds no copy of its own", (_path, source) => {
    const html = markup(source);
    const words = [...html.replace(/<[^>]*>/g, "\n").matchAll(/[A-Za-z]{2,}/g)].map((match) => match[0]);
    expect(words, "words between tags").toEqual([]);
    const attributes = [...html.matchAll(/\s(?:aria-label|title|placeholder|alt)="([^"]*)"/g)].map((match) => match[1]);
    expect(attributes, "literal aria-label, title, placeholder or alt").toEqual([]);
  });

  it("builds the sentences the person reads", () => {
    expect(strings.performer.autoResume(true)).toBe("Auto-resume: ON");
    expect(strings.performer.countInOnMarker(false)).toBe("Count-in when pressing marker: OFF");
    expect(strings.settings.milliseconds(strings.settings.fields.timeout)).toBe("Timeout (ms)");
  });
});
