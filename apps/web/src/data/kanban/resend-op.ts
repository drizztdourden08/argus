/* @layer renderer-app @kind logic */
import type { DocData } from '../api';
import { COPY_FIELDS } from './kanban.constants';
import { sendOp } from './send-op';
import type { OpOutcome, OwnerOp, SentOp } from './kanban.type';

const copiedFields = (op: OwnerOp): DocData => {
  const copied: Record<string, unknown> = {};
  for (const field of COPY_FIELDS) if (op[field] !== undefined) copied[field] = op[field];
  return copied;
};

const resendOp = (key: string, op: OwnerOp & OpOutcome, opId: string, rev: number | undefined): Promise<SentOp> =>
  sendOp(key, { ...copiedFields(op), prev: { ...op.prev, ...op.current }, baseRev: rev, force: true, retryOf: opId });

export { resendOp };
