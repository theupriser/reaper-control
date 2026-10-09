import { describe, expect, it } from "vitest";
import { keyAction, type KeyPress } from "./keyboard";

const press = (key: string, more: Partial<KeyPress> = {}): KeyPress => ({
  key,
  repeat: false,
  metaKey: false,
  ctrlKey: false,
  altKey: false,
  target: "BODY",
  editable: false,
  ...more,
});

describe("keyAction", () => {
  it("maps the performer keys", () => {
    expect(keyAction(press(" "), false)).toBe("PlayPause");
    expect(keyAction(press("a"), false)).toBe("ToggleAutoResume");
    expect(keyAction(press("ArrowLeft"), false)).toBe("Previous");
    expect(keyAction(press("ArrowRight"), false)).toBe("Next");
  });

  it("ignores keys while typing in a field", () => {
    for (const target of ["INPUT", "SELECT", "TEXTAREA"]) {
      expect(keyAction(press(" ", { target }), false)).toBeNull();
      expect(keyAction(press("a", { target }), false)).toBeNull();
    }
    expect(keyAction(press("a", { editable: true }), false)).toBeNull();
  });

  it("lets Space press a focused button instead of playing", () => {
    expect(keyAction(press(" ", { target: "BUTTON" }), false)).toBeNull();
    expect(keyAction(press("ArrowRight", { target: "BUTTON" }), false)).toBe("Next");
  });

  it("ignores repeats and shortcuts with a modifier", () => {
    expect(keyAction(press(" ", { repeat: true }), false)).toBeNull();
    expect(keyAction(press("a", { metaKey: true }), false)).toBeNull();
    expect(keyAction(press("a", { ctrlKey: true }), false)).toBeNull();
    expect(keyAction(press("a", { altKey: true }), false)).toBeNull();
  });

  it("leaves performer mode with Escape, only in performer mode", () => {
    expect(keyAction(press("Escape"), true)).toBe("ExitPerformer");
    expect(keyAction(press("Escape"), false)).toBeNull();
  });

  it("keeps Escape and the auto-resume key quiet while the screen is locked", () => {
    expect(keyAction(press("Escape"), true, true)).toBeNull();
    expect(keyAction(press("a"), true, true)).toBeNull();
    expect(keyAction(press(" "), true, true)).toBe("PlayPause");
    expect(keyAction(press("ArrowRight"), true, true)).toBe("Next");
    expect(keyAction(press("Escape"), true, false)).toBe("ExitPerformer");
  });
});
