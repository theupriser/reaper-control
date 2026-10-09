import { writable, type Readable } from "svelte/store";
import type { Command, LinkView, Notice } from "./generated/protocol";
import * as state from "./app-state";

/** What the store needs from the outside: the backend calls and its events. */
export interface Backend {
  dispatch(command: Command): Promise<void>;
  currentView(): Promise<LinkView>;
  currentProblem(): Promise<string | null>;
  onViewChange(handler: (view: LinkView) => void): Promise<() => void>;
  onNotice(handler: (notice: Notice) => void): Promise<() => void>;
  onLinkProblem(handler: (problem: string | null) => void): Promise<() => void>;
}

export interface AppStore extends Readable<state.AppState> {
  /** Sends a command; it is pending until the backend answers. */
  send(command: Command): Promise<void>;
  dismissNotice(key: string): void;
  /** Loads the current state and follows the backend; returns the function that stops following. */
  start(): () => void;
}

export function createAppStore(backend: Backend, now: () => number = Date.now): AppStore {
  const store = writable(state.initialState);
  const update = (change: (current: state.AppState) => state.AppState) => store.update(change);
  const fail = (error: unknown) => update((current) => state.withError(current, String(error)));

  return {
    subscribe: store.subscribe,
    async send(command) {
      update((current) => state.sent(current, command));
      try {
        await backend.dispatch(command);
        update((current) => state.acknowledged(current, command));
      } catch (error) {
        update((current) => state.rejected(current, command, String(error)));
      }
    },
    dismissNotice: (key) => update((current) => state.withoutNotice(current, key)),
    start() {
      const stops = [
        backend.onViewChange((link) => update((current) => state.withLink(current, link))),
        backend.onNotice((notice) => update((current) => state.withNotice(current, notice, now()))),
        backend.onLinkProblem((problem) => update((current) => state.withProblem(current, problem))),
      ];
      backend.currentView().then((link) => update((current) => state.withLink(current, link)), fail);
      backend.currentProblem().then((problem) => update((current) => state.withProblem(current, problem)), fail);
      const sweep = setInterval(() => update((current) => state.withExpiredNotices(current, now())), 500);
      return () => {
        stops.forEach((stop) => stop.then((unlisten) => unlisten()));
        clearInterval(sweep);
      };
    },
  };
}
