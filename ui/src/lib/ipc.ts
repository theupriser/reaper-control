import { invoke } from "@tauri-apps/api/core";
import type { AppState, Command } from "./types";

export const dispatch = (command: Command): Promise<AppState> =>
  invoke<AppState>("dispatch", { command });

export const currentState = (): Promise<AppState> => invoke<AppState>("current_state");
