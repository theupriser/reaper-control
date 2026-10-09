import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Command, InstallationView, LinkView, Notice, SetlistTransferView, Settings, SettingsView, SystemStats } from "./generated/protocol";

export const dispatch = (command: Command): Promise<void> => invoke<void>("dispatch", { command });

export const currentView = (): Promise<LinkView> => invoke<LinkView>("current_view");

export const currentProblem = (): Promise<string | null> => invoke<string | null>("current_problem");

export const onViewChange = (handler: (view: LinkView) => void): Promise<UnlistenFn> =>
  listen<LinkView>("link-view", (event) => handler(event.payload));

export const onNotice = (handler: (notice: Notice) => void): Promise<UnlistenFn> =>
  listen<Notice>("notice", (event) => handler(event.payload));

export const onLinkProblem = (handler: (problem: string | null) => void): Promise<UnlistenFn> =>
  listen<string | null>("link-problem", (event) => handler(event.payload));

export const currentSystemStats = (): Promise<SystemStats> => invoke<SystemStats>("current_system_stats");

export const currentSettings = (): Promise<SettingsView> => invoke<SettingsView>("current_settings");

export const saveSettings = (settings: Settings): Promise<void> => invoke<void>("save_settings", { settings });

export const currentInstallation = (): Promise<InstallationView> => invoke<InstallationView>("current_installation");

export const installExtension = (): Promise<InstallationView> => invoke<InstallationView>("install_extension");

export const currentTransfer = (): Promise<SetlistTransferView> => invoke<SetlistTransferView>("current_transfer");

export const restoreSetlists = (): Promise<number> => invoke<number>("restore_setlists");

export const importSetlists = (ids: string[]): Promise<number> => invoke<number>("import_setlists", { ids });

export const exportDiagnostics = (): Promise<string> => invoke<string>("export_diagnostics");

export const backend = { dispatch, currentView, currentProblem, onViewChange, onNotice, onLinkProblem };
