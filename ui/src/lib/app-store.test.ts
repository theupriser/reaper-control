import { describe, expect, it, vi } from "vitest";
import { get } from "svelte/store";
import { createAppStore, type Backend } from "./app-store";
import { emptyLink } from "./app-state";
import type { LinkProblem, LinkView, Notice } from "./generated/protocol";

function fakeBackend(overrides: Partial<Backend> = {}) {
  const handlers: { view?: (v: LinkView) => void; notice?: (n: Notice) => void; problem?: (p: LinkProblem | null) => void } = {};
  const backend: Backend = {
    dispatch: vi.fn().mockResolvedValue(undefined),
    currentView: vi.fn().mockResolvedValue(emptyLink),
    currentProblem: vi.fn().mockResolvedValue(null),
    onViewChange: async (h) => ((handlers.view = h), () => {}),
    onNotice: async (h) => ((handlers.notice = h), () => {}),
    onLinkProblem: async (h) => ((handlers.problem = h), () => {}),
    ...overrides,
  };
  return { backend, handlers };
}

describe("app store", () => {
  it("marks a command pending until the backend answers", async () => {
    let answer!: () => void;
    const { backend } = fakeBackend({ dispatch: () => new Promise<void>((resolve) => (answer = resolve)) });
    const store = createAppStore(backend);
    const done = store.send("Play");
    expect(get(store).pending).toEqual(["Play"]);
    answer();
    await done;
    expect(get(store).pending).toEqual([]);
  });

  it("keeps the error of a rejected command and clears it on the next success", async () => {
    const dispatch = vi.fn().mockRejectedValueOnce("no link").mockResolvedValue(undefined);
    const store = createAppStore(fakeBackend({ dispatch }).backend);
    await store.send("Next");
    expect(get(store)).toMatchObject({ error: "no link", pending: [] });
    await store.send("Next");
    expect(get(store).error).toBeNull();
  });

  it("names a command with data by its variant", async () => {
    let answer!: () => void;
    const store = createAppStore(fakeBackend({ dispatch: () => new Promise<void>((r) => (answer = r)) }).backend);
    const done = store.send({ Seek: { position: 3, count_in: false } });
    expect(get(store).pending).toEqual(["Seek"]);
    answer();
    await done;
  });

  it("follows the backend events and drops a dismissed notice", async () => {
    const { backend, handlers } = fakeBackend();
    const store = createAppStore(backend, () => 0);
    const stop = store.start();
    await Promise.resolve();
    handlers.problem?.({ message: "Wrong token", extension_outdated: false });
    handlers.notice?.({ key: "a", level: "Info", title: "Hi", text: "There" });
    expect(get(store).problem?.message).toBe("Wrong token");
    expect(get(store).notices).toHaveLength(1);
    store.dismissNotice("a");
    expect(get(store).notices).toEqual([]);
    stop();
  });
});
