import { describe, expect, it } from "vitest";
import type { AppState } from "./types";

describe("AppState", () => {
  it("starts idle", () => {
    const state: AppState = { phase: "Idle" };
    expect(state.phase).toBe("Idle");
  });
});
