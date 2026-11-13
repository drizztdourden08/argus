/* @layer renderer-app @kind logic */
import { apiRequest } from '../api';
import type { BoardInfo } from './boards.type';

const state: { boards: BoardInfo[]; started: boolean; listeners: Set<() => void> } = {
  boards: [],
  started: false,
  listeners: new Set(),
};

const publish = (boards: BoardInfo[]): void => {
  state.boards = boards;
  state.listeners.forEach((listener) => listener());
};

const refresh = async (): Promise<void> => {
  state.started = true;
  try {
    publish(await apiRequest<BoardInfo[]>('/api/boards'));
  } catch {
    state.started = state.boards.length > 0;
  }
};

const boardsStore = {
  get: (): BoardInfo[] => state.boards,
  subscribe: (listener: () => void): (() => void) => {
    state.listeners.add(listener);
    return () => state.listeners.delete(listener);
  },
  ensure: (): void => {
    if (!state.started) void refresh();
  },
  refresh,
  set: publish,
};

export { boardsStore };
