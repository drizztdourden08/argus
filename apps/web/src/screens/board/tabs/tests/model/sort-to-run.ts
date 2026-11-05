/* @layer renderer-app @kind logic */
import type { SortDirection } from '@drizztdourden08/tessera';
import type { TestDoc, TestOrder } from './model.type';
import { isNatural } from './sort-direction';

const byRank = (a: TestDoc, b: TestDoc): number =>
  (a.rank ?? 0) - (b.rank ?? 0) || a.createdAt.localeCompare(b.createdAt) || a.id.localeCompare(b.id);

const newestFirst = (a: TestDoc, b: TestDoc): number => String(b.updatedAt).localeCompare(String(a.updatedAt));

const sortToRun = (tests: readonly TestDoc[], order: TestOrder = 'rank', direction: SortDirection | null = null): TestDoc[] => {
  const sorted = [...tests].sort(order === 'rank' ? byRank : newestFirst);
  return isNatural(order, direction) ? sorted : sorted.reverse();
};

export { sortToRun };
