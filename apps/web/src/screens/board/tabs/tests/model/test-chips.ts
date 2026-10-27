/* @layer renderer-app @kind logic */
import type { TestChipSet, TestDoc, TestGroup } from './model.type';

const testChips = (test: TestDoc, group: TestGroup, conflicted: ReadonlySet<string>): TestChipSet => ({
  item: group !== 'item' ? test.item ?? '' : '',
  thread: group !== 'thread' ? test.thread ?? '' : '',
  auto: test.mode === 'auto',
  conflict: conflicted.has(test.id),
  archived: Boolean(test.archivedAt),
});

export { testChips };
