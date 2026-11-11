/* @layer renderer-app @kind logic */
import type { CompareRow } from '../../../../data/types';
import { QUIET_STATUSES } from '../compare.constants';
import type { RowFilter } from '../compare.type';
import { cellsOf } from './cells-of';

const isSame = (row: CompareRow): boolean => cellsOf(row).every((cell) => cell.status === undefined || QUIET_STATUSES.has(cell.status));

const isArchivedEverywhere = (row: CompareRow): boolean => {
  const present = cellsOf(row).filter((cell) => cell.present);
  return present.length > 0 && present.every((cell) => cell.archived === true);
};

const chatShown = (row: CompareRow, filter: RowFilter): boolean =>
  (filter.showSame || !isSame(row)) && (filter.showArchived || !isArchivedEverywhere(row));

const keepWithParents = (row: CompareRow, byId: ReadonlyMap<string, CompareRow>, keep: Set<string>): void => {
  let cur: CompareRow | undefined = row;
  while (cur && !keep.has(cur.id)) {
    keep.add(cur.id);
    cur = cur.parentId ? byId.get(cur.parentId) : undefined;
  }
};

const visibleRows = (rows: readonly CompareRow[], filter: RowFilter): CompareRow[] => {
  const byId = new Map(rows.map((row) => [row.id, row]));
  const keep = new Set<string>();
  for (const row of rows) {
    if (row.kind === 'chat' && chatShown(row, filter)) keepWithParents(row, byId, keep);
    else if (row.kind === 'folder' && (filter.showSame || !isSame(row))) keep.add(row.id);
  }
  return rows.filter((row) => keep.has(row.id));
};

export { visibleRows };
