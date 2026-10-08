import { describe, expect, it } from "vitest";
import type { AppState } from "./generated/protocol";

describe("AppState", () => {
  it("starts idle", () => {
    const state: AppState = {
      phase: "Idle",
      position: 0,
      auto_resume: true,
      count_in_on_marker: false,
      record_armed: false,
      current_song: null,
    };
    expect(state.phase).toBe("Idle");
  });
});
