/* @layer renderer-app @kind logic */
import { toMillis } from '../../../../../format';
import type { CheckRun } from '../../../../../data/remote';

const stamp = (run: CheckRun): number => {
  const at = toMillis(run.startedAt);
  return Number.isFinite(at) ? at : 0;
};

const sortRuns = (runs: readonly CheckRun[]): CheckRun[] => [...runs].sort((left, right) => stamp(right) - stamp(left));

export { sortRuns };
