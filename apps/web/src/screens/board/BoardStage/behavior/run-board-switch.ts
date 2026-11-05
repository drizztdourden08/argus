/* @layer renderer-app @kind logic */
import { ENTER_MS, LEAVE_MS, READY_WAIT_MS } from '../BoardStage.constants';
import type { SwitchAction, SwitchPhase } from '../BoardStage.type';

interface SwitchStep {
  phase: SwitchPhase;
  ready: boolean;
  reduced: boolean;
  dispatch: (action: SwitchAction) => void;
}

const after = (ms: number, run: () => void): (() => void) => {
  if (ms <= 0) {
    run();
    return () => undefined;
  }
  const timer = setTimeout(run, ms);
  return () => clearTimeout(timer);
};

const runBoardSwitch = (step: SwitchStep): (() => void) => {
  const { phase, ready, reduced, dispatch } = step;
  if (phase === 'leaving') return after(reduced ? 0 : LEAVE_MS, () => dispatch({ type: 'left' }));
  if (phase === 'swap') return after(ready || reduced ? 0 : READY_WAIT_MS, () => dispatch({ type: 'revealed' }));
  if (phase === 'entering') return after(reduced ? 0 : ENTER_MS, () => dispatch({ type: 'entered' }));
  return () => undefined;
};

export { runBoardSwitch };
