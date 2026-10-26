/* @layer renderer-app @kind logic */
import { sendOp } from './send-op';
import type { EditChange, EditRef, SentOp } from './kanban.type';

const editDoc = (key: string, ref: EditRef, change: EditChange): Promise<SentOp> =>
  sendOp(key, {
    kind: 'edit',
    target: { c: ref.c, id: ref.id },
    set: { [change.field]: change.value },
    prev: { [change.field]: change.prev },
    baseRev: ref.rev,
  });

export { editDoc };
