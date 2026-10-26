/* @layer renderer-app @kind logic */
import { compareIds } from './compare-ids';
import type { TestDoc } from './model.type';

const sortToRun = (tests: readonly TestDoc[]): TestDoc[] =>
  [...tests].sort((a, b) => (a.rank ?? 0) - (b.rank ?? 0) || compareIds(a.id, b.id));

export { sortToRun };
