/* @layer renderer-app @kind logic */
import { sendOp } from './send-op';
import type { CriterionOp, ItemRef, SentOp } from './kanban.type';

const criterionOp = (key: string, item: ItemRef, op: CriterionOp): Promise<SentOp> =>
  sendOp(key, {
    kind: 'ac',
    target: { c: 'items', id: item.id },
    do: op.do,
    n: op.n,
    to: op.to,
    text: op.text,
    prev: op.prev === undefined ? undefined : { text: op.prev },
    baseRev: item.rev,
  });

export { criterionOp };
