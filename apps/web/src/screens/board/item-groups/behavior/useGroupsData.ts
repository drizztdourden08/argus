/* @layer renderer-app @kind logic */
import type { Item, ThreadChoice } from '../../../../data/kanban';
import { useCollection } from '../../../../data/react';
import type { GroupsData } from '../item-groups.type';

const useGroupsData = (boardKey: string): GroupsData => {
  const items = useCollection<Item>(boardKey, 'items');
  const threads = useCollection<ThreadChoice>(boardKey, 'threads');
  return { ready: items.ready && threads.ready, items: items.docs, threads: threads.docs };
};

export { useGroupsData };
