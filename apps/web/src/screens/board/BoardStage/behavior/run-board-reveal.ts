/* @layer renderer-app @kind logic */
import { READY_WAIT_MS } from '../BoardStage.constants';
import type { BoardReveal } from '../BoardStage.type';

const runBoardReveal = (step: BoardReveal): (() => void) => {
  const { ready, reduced, reveal } = step;
  if (ready || reduced) {
    reveal();
    return () => undefined;
  }
  const timer = setTimeout(reveal, READY_WAIT_MS);
  return () => clearTimeout(timer);
};

export { runBoardReveal };
