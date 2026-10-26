/* @layer renderer-app @kind logic */
import { sendOp } from './send-op';
import type { SentOp, SoftKind } from './kanban.type';

const softOp = (key: string, kind: SoftKind, id: string, undoes?: string): Promise<SentOp> =>
  sendOp(key, { kind, target: { c: 'items', id }, undoes });

export { softOp };
