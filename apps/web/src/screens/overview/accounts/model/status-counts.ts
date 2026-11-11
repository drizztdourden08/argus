/* @layer renderer-app @kind logic */
import type { CompareRow, CompareStatus } from '../../../../data/types';
import { QUIET_STATUSES } from '../compare.constants';
import { cellsOf } from './cells-of';

const statusCounts = (rows: readonly CompareRow[]): Map<CompareStatus, number> => {
  const counts = new Map<CompareStatus, number>();
  for (const row of rows) {
    for (const cell of cellsOf(row)) {
      if (!cell.status || QUIET_STATUSES.has(cell.status)) continue;
      counts.set(cell.status, (counts.get(cell.status) ?? 0) + 1);
    }
  }
  return counts;
};

export { statusCounts };
