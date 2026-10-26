/* @layer renderer-app @kind logic */
import type { DocData } from '../../../../../data/api';
import type { ItemRef } from '../../../../../data/kanban';

const bulkMoves = (items: readonly ItemRef[], to: { status: string } | { thread: string }): DocData[] =>
  items.map((item) => ({
    kind: 'move',
    target: { c: 'items', id: item.id },
    to,
    prev: { thread: item.thread, status: item.status },
    baseRev: item.rev,
  }));

export { bulkMoves };
