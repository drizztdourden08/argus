/* @layer renderer-app @kind logic */
import type { ItemDoc, TestDoc } from './model.type';

const followItemThread = (tests: readonly TestDoc[], items: readonly ItemDoc[]): TestDoc[] => {
  const threadOf = new Map(items.map((item) => [item.id, item.thread]));
  return tests.map((test) => {
    const thread = test.item ? threadOf.get(test.item) : undefined;
    return thread && test.thread !== thread ? { ...test, thread } : test;
  });
};

export { followItemThread };
