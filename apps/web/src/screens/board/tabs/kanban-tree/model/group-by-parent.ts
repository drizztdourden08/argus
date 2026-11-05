/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';
import { byRank } from './by-rank';

const groupByParent = (items: readonly ItemRef[]): Map<string, ItemRef[]> => {
  const groups = new Map<string, ItemRef[]>();
  for (const item of byRank(items)) {
    if (item.parent) groups.set(item.parent, [...(groups.get(item.parent) ?? []), item]);
  }
  return groups;
};

export { groupByParent };
