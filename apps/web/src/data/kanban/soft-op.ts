/* @layer renderer-app @kind logic */
import { sendOp } from './send-op';
import type { SentOp, SoftRequest } from './kanban.type';

const softOp = (key: string, request: SoftRequest): Promise<SentOp> =>
  sendOp(key, { kind: request.kind, target: { c: request.c ?? 'items', id: request.id }, undoes: request.undoes });

export { softOp };
