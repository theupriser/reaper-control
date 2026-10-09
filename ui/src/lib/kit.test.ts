import { describe, expect, it } from "vitest";

const sources: Record<string, string> = import.meta.glob("../kit/*.svelte", {
  query: "?raw",
  import: "default",
  eager: true,
});

describe("component kit", () => {
  it("has the ten components", () => {
    const names = Object.keys(sources).map((path) => path.split("/").pop());
    expect(names.sort()).toEqual(["Button.svelte", "Clock.svelte", "Dialog.svelte", "ListRow.svelte", "Panel.svelte", "StatusBadge.svelte", "Switch.svelte", "Timeline.svelte", "Toast.svelte", "Toggle.svelte"]);
  });

  it.each(Object.entries(sources))("%s takes its sizes from tokens, not px values", (_path, source) => {
    const style = source.match(/<style>[\s\S]*<\/style>/)?.[0] ?? "";
    expect(style).not.toMatch(/font-size:\s*\d+px/);
    expect(style).not.toMatch(/(?:padding|gap):[^;]*\b[1-9]\d*px/);
  });
});
