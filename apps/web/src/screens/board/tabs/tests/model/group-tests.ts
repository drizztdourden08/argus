/* @layer renderer-app @kind logic */
import { compareIds } from './compare-ids';
import { NO_KEY } from './model.constants';
import type { GroupCard, ItemDoc, TestDoc, TestGroup } from './model.type';
import { tally } from './tally';

const keyOf = (test: TestDoc, group: TestGroup): string => {
  const key = group === 'item' ? test.item : test.thread;
  if (key) return key;
  return NO_KEY;
};

const groupTests = (tests: readonly TestDoc[], group: TestGroup, items: readonly ItemDoc[]): GroupCard[] => {
  const slots = new Map<string, TestDoc[]>();
  for (const test of tests) {
    const key = keyOf(test, group);
    slots.set(key, [...(slots.get(key) ?? []), test]);
  }
  return [...slots]
    .sort(([a], [b]) => compareIds(a, b))
    .map(([key, rows]) => ({ key, item: group === 'item' ? items.find((item) => item.id === key) ?? null : null, tests: rows, counts: tally(rows) }));
};

export { groupTests };
