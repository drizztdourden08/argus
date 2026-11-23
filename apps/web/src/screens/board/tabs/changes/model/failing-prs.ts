/* @layer renderer-app @kind logic */
import type { CheckRun } from '../../../../../data/remote';
import { checksProgress } from './checks-progress';

const failingPrIds = (ids: readonly string[], runs: readonly (readonly CheckRun[] | null)[]): Set<string> =>
  new Set(ids.filter((_, index) => checksProgress(runs[index] ?? null)?.failed));

export { failingPrIds };
