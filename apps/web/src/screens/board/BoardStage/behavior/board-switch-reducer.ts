/* @layer renderer-app @kind logic */
import type { SwitchAction, SwitchState } from '../BoardStage.type';

const boardSwitchReducer = (state: SwitchState, action: SwitchAction): SwitchState => {
  switch (action.type) {
    case 'request':
      if (action.key === state.target) return state;
      if (action.key === state.shown && state.phase === 'idle') return state;
      return { shown: state.shown, target: action.key, phase: 'leaving' };
    case 'left':
      return state.phase === 'leaving' ? { shown: state.target, target: state.target, phase: 'swap' } : state;
    case 'revealed':
      return state.phase === 'swap' ? { ...state, phase: 'entering' } : state;
    case 'entered':
      return state.phase === 'entering' ? { ...state, phase: 'idle' } : state;
  }
};

export { boardSwitchReducer };
