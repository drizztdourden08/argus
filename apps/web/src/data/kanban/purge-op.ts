/* @layer renderer-app @kind logic */
import { sendOp } from './send-op';
import type { PurgeRequest, SentOp } from './kanban.type';

const purgeOp = (key: string, request: PurgeRequest): Promise<SentOp> =>
  sendOp(key, { kind: 'purge', criteria: request.criteria, expect: request.expect, confirm: request.confirm });

export { purgeOp };
