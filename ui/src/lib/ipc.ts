import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { ChecklistView, Command, InstallationView, LinkProblem, LinkView, Notice, SetlistTransferView, Settings, SettingsView, SystemStats } from "./generated/protocol";

export const dispatch = (command: Command): Promise<void> => invoke<void>("dispatch", { command });

export const bundledExtensionVersion = (): Promise<string> => invoke<string>("bundled_extension_version");

export const currentView = (): Promise<LinkView> => invoke<LinkView>("current_view");

export const currentProblem = (): Promise<LinkProblem | null> => invoke<LinkProblem | null>("current_problem");

export const onViewChange = (handler: (view: LinkView) => void): Promise<UnlistenFn> =>
  listen<LinkView>("link-view", (event) => handler(event.payload));

export const onNotice = (handler: (notice: Notice) => void): Promise<UnlistenFn> =>
  listen<Notice>("notice", (event) => handler(event.payload));

export const onLinkProblem = (handler: (problem: LinkProblem | null) => void): Promise<UnlistenFn> =>
  listen<LinkProblem | null>("link-problem", (event) => handler(event.payload));

export const currentSystemStats = (): Promise<SystemStats> => invoke<SystemStats>("current_system_stats");

export const currentSettings = (): Promise<SettingsView> => invoke<SettingsView>("current_settings");

export const saveSettings = (settings: Settings): Promise<void> => invoke<void>("save_settings", { settings });

export const currentInstallation = (): Promise<InstallationView> => invoke<InstallationView>("current_installation");

export const installExtension = (): Promise<InstallationView> => invoke<InstallationView>("install_extension");

export const currentChecklist = (): Promise<ChecklistView> => invoke<ChecklistView>("current_checklist");

export const currentTransfer = (): Promise<SetlistTransferView> => invoke<SetlistTransferView>("current_transfer");

export const restoreSetlists = (): Promise<number> => invoke<number>("restore_setlists");

export const importSetlists = (ids: string[]): Promise<number> => invoke<number>("import_setlists", { ids });

export const exportDiagnostics = (): Promise<string> => invoke<string>("export_diagnostics");

export const backend = { dispatch, currentView, currentProblem, onViewChange, onNotice, onLinkProblem };
