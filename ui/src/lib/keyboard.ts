import { keyIntent, type KeyIntent } from "./performer";

export interface KeyPress {
  key: string;
  repeat: boolean;
  metaKey: boolean;
  ctrlKey: boolean;
  altKey: boolean;
  /** Tag name of the focused element, upper case. */
  target: string;
  /** Whether the focused element accepts typing (contenteditable). */
  editable: boolean;
}

export type KeyAction = KeyIntent | "ExitPerformer";

const TYPING = new Set(["INPUT", "SELECT", "TEXTAREA"]);

export function keyAction(press: KeyPress, performerMode: boolean): KeyAction | null {
  if (press.repeat || press.metaKey || press.ctrlKey || press.altKey) return null;
  if (press.editable || TYPING.has(press.target)) return null;
  if (press.key === "Escape") return performerMode ? "ExitPerformer" : null;
  // Space on a focused button presses that button, not play.
  if (press.key === " " && press.target === "BUTTON") return null;
  return keyIntent(press.key);
}

export function toKeyPress(event: KeyboardEvent): KeyPress {
  const element = event.target instanceof HTMLElement ? event.target : null;
  return {
    key: event.key,
    repeat: event.repeat,
    metaKey: event.metaKey,
    ctrlKey: event.ctrlKey,
    altKey: event.altKey,
    target: element?.tagName ?? "",
    editable: element?.isContentEditable ?? false,
  };
}
