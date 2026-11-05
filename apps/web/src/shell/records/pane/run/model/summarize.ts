/* @layer renderer-app @kind logic */
import { EMPTY_SUMMARY } from '../run.constants';
import type { PlanRun, RunSummary } from '../run.type';
import { lastResult } from './last-result';

const summarize = (run: PlanRun, ids: readonly string[]): RunSummary => {
  const counts: RunSummary = { ...EMPTY_SUMMARY, total: ids.length };
  ids.forEach((id) => {
    const status = lastResult(run, id)?.status;
    counts[status ?? 'notRun'] += 1;
  });
  return counts;
};

export { summarize };
