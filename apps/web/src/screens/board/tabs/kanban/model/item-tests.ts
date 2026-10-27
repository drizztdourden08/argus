/* @layer renderer-app @kind logic */
import type { TestScope } from './model.type';

const itemTests = <T extends TestScope>(tests: readonly T[], closed: ReadonlySet<string>, archived: boolean): T[] =>
  tests.filter((test) => !test.deletedAt && (archived || !test.archivedAt) && !(test.build && closed.has(test.build)));

export { itemTests };
