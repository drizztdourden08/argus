/* @layer renderer-app @kind logic */
import type { TestDoc } from './model.type';

const visibleTests = (tests: readonly TestDoc[], archived = false): TestDoc[] =>
  tests.filter((test) => !test.deletedAt && (archived || !test.archivedAt));

export { visibleTests };
