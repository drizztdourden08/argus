/* @layer renderer-app @kind logic */
import type { PlanRun } from '../run.type';
import { lastResult } from './last-result';

const firstOpen = (run: PlanRun, ids: readonly string[]): number => {
  const index = ids.findIndex((id) => lastResult(run, id) === null);
  return index === -1 ? 0 : index;
};

export { firstOpen };
