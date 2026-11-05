/* @layer renderer-app @kind logic */
import type { DocData } from '../api';
import { BOARDS_COLLECTION } from './boards.constants';
import type { BoardOp } from './boards.type';

const targetOf = (id: string) => ({ c: BOARDS_COLLECTION, id });

const boardEntry = (op: BoardOp): DocData => {
  if (op.kind !== 'edit') return { kind: op.kind, target: targetOf(op.id) };
  return { kind: 'edit', target: targetOf(op.id), set: { [op.field]: op.value }, prev: { [op.field]: op.prev }, baseRev: op.baseRev };
};

export { boardEntry };
