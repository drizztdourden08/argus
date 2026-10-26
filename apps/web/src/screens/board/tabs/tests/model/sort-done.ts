/* @layer renderer-app @kind logic */
import { compareIds } from './compare-ids';
import type { TestDoc } from './model.type';

const doneAt = (test: TestDoc): string => test.result?.at ?? test.updatedAt;

const sortDone = (tests: readonly TestDoc[]): TestDoc[] =>
  [...tests].sort((a, b) => doneAt(b).localeCompare(doneAt(a)) || compareIds(b.id, a.id));

export { sortDone };
