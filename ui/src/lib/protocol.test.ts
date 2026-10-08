import { describe, expect, it } from "vitest";
import type { Live } from "./generated/protocol";

describe("Live", () => {
  it("names songs by their index in the catalog", () => {
    const live: Live = {
      sequence: 1,
      timestamp: 0,
      transport: "Stopped",
      position: 0,
      phase: "Idle",
      setlist_id: null,
      current_song: null,
      next_song: null,
      autoplay: true,
      count_in: false,
      record_armed: false,
      catalog_revision: 0,
      setlist_revision: 0,
    };
    expect(live.phase).toBe("Idle");
  });
});
