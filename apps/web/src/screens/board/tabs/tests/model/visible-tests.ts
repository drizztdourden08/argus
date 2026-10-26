/* @layer renderer-app @kind logic */
import type { TestDoc } from './model.type';

const visibleTests = (tests: readonly TestDoc[]): TestDoc[] => tests.filter((test) => !test.deletedAt && !test.archivedAt);

export { visibleTests };
