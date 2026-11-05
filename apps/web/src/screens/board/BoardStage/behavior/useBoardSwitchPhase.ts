/* @layer renderer-app @kind logic */
import { useEffect, useReducer } from 'react';
import type { SwitchState } from '../BoardStage.type';
import { boardSwitchReducer } from './board-switch-reducer';
import { prefersReducedMotion } from './prefers-reduced-motion';
import { runBoardSwitch } from './run-board-switch';

const useBoardSwitchPhase = (boardKey: string, ready: boolean): SwitchState => {
  const [state, dispatch] = useReducer(boardSwitchReducer, { shown: boardKey, target: boardKey, phase: 'idle' });
  useEffect(() => dispatch({ type: 'request', key: boardKey }), [boardKey]);
  useEffect(
    () => runBoardSwitch({ phase: state.phase, ready, reduced: prefersReducedMotion(), dispatch }),
    [state.phase, state.target, ready],
  );
  return state;
};

export { useBoardSwitchPhase };
