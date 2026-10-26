/* @layer renderer-app @kind logic */
import { sendOp } from './send-op';
import type { EditField, ItemRef, SentOp } from './kanban.type';

const editItem = (key: string, item: ItemRef, field: EditField, value: string): Promise<SentOp> =>
  sendOp(key, {
    kind: 'edit',
    target: { c: 'items', id: item.id },
    set: { [field]: value },
    prev: { [field]: item[field] ?? '' },
    baseRev: item.rev,
  });

export { editItem };
