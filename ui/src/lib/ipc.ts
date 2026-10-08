import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Command, LinkView } from "./generated/protocol";

export const dispatch = (command: Command): Promise<void> => invoke<void>("dispatch", { command });

export const currentView = (): Promise<LinkView> => invoke<LinkView>("current_view");

export const onViewChange = (handler: (view: LinkView) => void): Promise<UnlistenFn> =>
  listen<LinkView>("link-view", (event) => handler(event.payload));
