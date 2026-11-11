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

const headersOf = (rows: readonly CompareRow[]): Map<string, string> => {
  const headers = new Map<string, string>();
  const above: string[] = [];
  for (const row of rows) {
    above.length = row.depth;
    const header = row.parentId ?? above[row.depth - 1];
    if (header) headers.set(row.id, header);
    above[row.depth] = row.id;
  }
  return headers;
};

const keepWithParents = (id: string, headers: ReadonlyMap<string, string>, keep: Set<string>): void => {
  let cur: string | undefined = id;
  while (cur && !keep.has(cur)) {
    keep.add(cur);
    cur = headers.get(cur);
  }
};

const keptIds = (rows: readonly CompareRow[], headers: ReadonlyMap<string, string>, filter: RowFilter): Set<string> => {
  const keep = new Set<string>();
  for (const row of rows) {
    if (row.kind === 'chat' && chatShown(row, filter)) keepWithParents(row.id, headers, keep);
    else if (row.kind === 'folder' && (filter.showSame || !isSame(row))) keep.add(row.id);
  }
  return keep;
};

const visibleRows = (rows: readonly CompareRow[], filter: RowFilter): CompareRow[] => {
  const keep = keptIds(rows, headersOf(rows), filter);
  return rows.filter((row) => keep.has(row.id));
};

export { visibleRows };
