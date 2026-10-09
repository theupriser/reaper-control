import { strings } from "./strings";

export type ScreenId = "player" | "setlists" | "checklist" | "settings" | "help";

export interface ScreenDef {
  id: ScreenId;
  label: string;
  icon: string;
}

export const screens: ScreenDef[] = [
  { id: "player", label: strings.screens.player, icon: "M5 3l14 9-14 9V3z" },
  { id: "setlists", label: strings.screens.setlists, icon: "M8 6h13M8 12h13M8 18h13M3 6h.01M3 12h.01M3 18h.01" },
  {
    id: "checklist",
    label: strings.screens.checklist,
    icon: "M9 11l3 3L22 4M21 12v7a2 2 0 01-2 2H5a2 2 0 01-2-2V5a2 2 0 012-2h11",
  },
  {
    id: "settings",
    label: strings.screens.settings,
    icon: "M4 21v-7M4 10V3M12 21v-9M12 8V3M20 21v-5M20 12V3M1 14h6M9 8h6M17 16h6",
  },
  {
    id: "help",
    label: strings.screens.help,
    icon: "M12 22a10 10 0 100-20 10 10 0 000 20zM9.1 9a3 3 0 015.8 1c0 2-3 3-3 3M12 17h.01",
  },
];

export const screenLabel = (id: ScreenId): string =>
  screens.find((s) => s.id === id)?.label ?? id;
