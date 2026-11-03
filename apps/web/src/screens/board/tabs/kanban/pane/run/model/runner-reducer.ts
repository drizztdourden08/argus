/* @layer renderer-app @kind logic */
import type { RunnerAction, RunnerState } from '../run.type';

const clamp = (index: number, total: number): number => Math.max(0, Math.min(index, total - 1));

const moved = (cursor: number): RunnerState => ({ cursor, note: '', asked: null });

const runnerReducer = (state: RunnerState, action: RunnerAction): RunnerState => {
  switch (action.type) {
    case 'goto':
      return moved(clamp(action.index, action.total));
    case 'step':
      return moved(clamp(state.cursor + action.by, action.total));
    case 'recorded':
      return moved(clamp(state.cursor + 1, action.total));
    case 'note':
      return { ...state, note: action.text, asked: null };
    case 'ask':
      return { ...state, asked: action.status };
  }
};

export { runnerReducer };
