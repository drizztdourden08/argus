/* @layer renderer-app @kind logic */
import type { CheckRun } from '../../../../../data/remote';
import type { ChecksProgress } from '../changes.type';
import { sortRuns } from './sort-runs';
import { stepStatus } from './step-status';

const checksProgress = (runs: readonly CheckRun[] | null): ChecksProgress | null => {
  const newest = runs ? sortRuns(runs)[0] : undefined;
  if (!newest || newest.steps.length === 0) return null;
  const statuses = newest.steps.map(stepStatus);
  return { passed: statuses.filter((status) => status === 'pass').length, total: statuses.length, failed: newest.status === 'fail' || statuses.includes('fail') };
};

export { checksProgress };
