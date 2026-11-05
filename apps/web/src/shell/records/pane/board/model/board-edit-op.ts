/* @layer renderer-app @kind logic */
import type { BoardOp } from '../../../../../data/boards';
import type { BoardRecord } from '../../../../../screens/overview/boards/model';
import type { BoardField } from '../board.type';

const boardEditOp = (record: BoardRecord, field: BoardField, value: string): BoardOp => ({
  kind: 'edit',
  id: record.id,
  field,
  value,
  prev: record[field] === '' ? null : record[field],
  baseRev: record.rev,
});

export { boardEditOp };
