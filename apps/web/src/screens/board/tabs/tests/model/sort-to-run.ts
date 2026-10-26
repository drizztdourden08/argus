/* @layer renderer-app @kind logic */
import type { TestDoc, TestOrder } from './model.type';

const byRank = (a: TestDoc, b: TestDoc): number =>
  (a.rank ?? 0) - (b.rank ?? 0) || a.createdAt.localeCompare(b.createdAt) || a.id.localeCompare(b.id);

const newestFirst = (a: TestDoc, b: TestDoc): number => String(b.updatedAt).localeCompare(String(a.updatedAt));

const sortToRun = (tests: readonly TestDoc[], order: TestOrder = 'rank'): TestDoc[] => [...tests].sort(order === 'rank' ? byRank : newestFirst);

export { sortToRun };
