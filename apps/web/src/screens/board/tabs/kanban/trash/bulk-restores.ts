/* @layer renderer-app @kind logic */
import { SOFT_INVERSE } from '../../../../../data/kanban';
import type { SoftKind } from '../../../../../data/kanban';
import type { BulkSofts } from '../model';
import type { TrashRow } from './trash.type';

const restoreKind = (row: TrashRow): SoftKind => (row.archived ? 'unarchive' : 'restore');

const bulkRestores = (rows: readonly TrashRow[]): BulkSofts => ({
  ops: rows.map((row) => ({ kind: restoreKind(row), target: { c: row.c, id: row.id } })),
  inverse: rows.map((row) => ({ kind: SOFT_INVERSE[restoreKind(row)], target: { c: row.c, id: row.id } })),
});

export { bulkRestores };
