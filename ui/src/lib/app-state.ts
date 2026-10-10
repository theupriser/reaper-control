import type { Command, LinkProblem, LinkView, Notice } from "./generated/protocol";
import { addNotice, dismissNotice, expireNotices, type ShownNotice } from "./notices";

export interface AppState {
  link: LinkView;
  problem: LinkProblem | null;
  notices: ShownNotice[];
  /** The last command that failed, shown until the next one succeeds. */
  error: string | null;
  /** Names of the commands sent and not yet acknowledged. */
  pending: string[];
}

export const emptyLink: LinkView = {
  status: "NotRunning",
  live: null,
  catalog: { revision: 0, setlist_revision: 0, project_id: "", songs: [], project_songs: [], cues: [], setlists: [], active_setlist: null },
};

export const initialState: AppState = { link: emptyLink, problem: null, notices: [], error: null, pending: [] };

export const commandName = (command: Command): string => (typeof command === "string" ? command : (Object.keys(command)[0] ?? ""));

const removeOne = (names: string[], name: string): string[] => {
  const at = names.indexOf(name);
  return at < 0 ? names : [...names.slice(0, at), ...names.slice(at + 1)];
};

export const withLink = (state: AppState, link: LinkView): AppState => ({ ...state, link });
export const withProblem = (state: AppState, problem: LinkProblem | null): AppState => ({ ...state, problem });
export const withNotice = (state: AppState, notice: Notice, now: number): AppState => ({ ...state, notices: addNotice(state.notices, notice, now) });
export const withoutNotice = (state: AppState, key: string): AppState => ({ ...state, notices: dismissNotice(state.notices, key) });
export const withExpiredNotices = (state: AppState, now: number): AppState => ({ ...state, notices: expireNotices(state.notices, now) });
export const withError = (state: AppState, error: string | null): AppState => ({ ...state, error });
export const sent = (state: AppState, command: Command): AppState => ({ ...state, pending: [...state.pending, commandName(command)] });
export const acknowledged = (state: AppState, command: Command): AppState => ({ ...state, error: null, pending: removeOne(state.pending, commandName(command)) });
export const rejected = (state: AppState, command: Command, error: string): AppState => ({ ...state, error, pending: removeOne(state.pending, commandName(command)) });
