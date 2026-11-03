/* @layer renderer-app @kind logic */
import { FLAGGED_STATUSES } from '../run.constants';
import type { PlanRun, RunResult } from '../run.type';
import { lastResult } from './last-result';

const flaggedResults = (run: PlanRun, ids: readonly string[]): RunResult[] =>
  ids.flatMap((id) => {
    const result = lastResult(run, id);
    return result && FLAGGED_STATUSES.includes(result.status) ? [result] : [];
  });

export { flaggedResults };
