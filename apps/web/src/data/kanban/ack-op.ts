/* @layer renderer-app @kind logic */
import { sendOp } from './send-op';
import type { SentOp } from './kanban.type';

const ackOp = (key: string, entryId: string): Promise<SentOp> =>
  sendOp(key, { kind: 'ack', replyTo: entryId, text: '' });

export { ackOp };
