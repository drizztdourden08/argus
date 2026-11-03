/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';
import type { ChildCount } from './model.type';

const childCounts = (items: readonly ItemRef[]): Map<string, ChildCount> => {
  const counts = new Map<string, ChildCount>();
  for (const item of items) {
    if (!item.parent || item.deletedAt || item.archivedAt || item.status === 'dropped') continue;
    const count = counts.get(item.parent) ?? { done: 0, total: 0 };
    counts.set(item.parent, { done: count.done + (item.status === 'done' ? 1 : 0), total: count.total + 1 });
  }
  return counts;
};

export { childCounts };
