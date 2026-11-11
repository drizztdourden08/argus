/* @layer renderer-app @kind logic */
import type { ReviewCell } from '../../../../../data/types';
import { DAY_MS } from '../review.constants';

const archivedNote = (cell: ReviewCell, now: number): string | null => {
  if (cell.archived !== true) return null;
  const at = cell.lastActivityAt ? new Date(cell.lastActivityAt).getTime() : Number.NaN;
  return Number.isNaN(at) ? 'archived' : `archived ${Math.max(0, Math.floor((now - at) / DAY_MS))} d`;
};

export { archivedNote };
