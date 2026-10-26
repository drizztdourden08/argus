/* @layer renderer-app @kind logic */
import { sendOp } from './send-op';
import type { ItemInput, SentOp } from './kanban.type';

const createItem = (key: string, item: ItemInput): Promise<SentOp> =>
  sendOp(key, { kind: 'create', target: { c: 'items' }, set: item });

export { createItem };
