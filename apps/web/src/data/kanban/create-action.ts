/* @layer renderer-app @kind logic */
import { sendOp } from './send-op';
import type { ActionInput, SentOp } from './kanban.type';

const createAction = (key: string, action: ActionInput): Promise<SentOp> =>
  sendOp(key, { kind: 'create', target: { c: 'actions' }, set: action });

export { createAction };
