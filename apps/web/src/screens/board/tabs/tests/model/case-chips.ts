/* @layer renderer-app @kind logic */
import { labelOf } from '../../../../../data/registry';
import type { RowChip } from '../../../../../shell';
import { AUTO_TIP } from './model.constants';
import type { TestDoc, TestGroup } from './model.type';
import { testChips } from './test-chips';

const caseChips = (test: TestDoc, group: TestGroup, conflicted: ReadonlySet<string>): RowChip[] => {
  const set = testChips(test, group, conflicted);
  const chips: RowChip[] = [];
  if (set.conflict) chips.push({ id: 'conflict', text: 'conflict', title: 'A change to this test was not taken' });
  if (set.item) chips.push({ id: 'item', text: set.item, title: `Item: ${set.item}`, code: true });
  if (set.thread) chips.push({ id: 'thread', text: set.thread, title: `Thread: ${set.thread}` });
  chips.push({ id: 'mode', text: labelOf('mode', set.auto ? 'auto' : 'manual'), title: set.auto ? AUTO_TIP : 'You run this one by hand' });
  if (set.archived) chips.push({ id: 'archived', text: 'archived' });
  return chips;
};

export { caseChips };
