/* @layer renderer-app @kind logic */
import { sendOp } from './send-op';
import type { ItemRef, MoveTarget, SentOp } from './kanban.type';

const moveItem = (key: string, item: ItemRef, to: MoveTarget): Promise<SentOp> =>
  sendOp(key, {
    kind: 'move',
    target: { c: 'items', id: item.id },
    to,
    prev: { thread: item.thread, status: item.status },
    baseRev: item.rev,
  });

export { moveItem };
