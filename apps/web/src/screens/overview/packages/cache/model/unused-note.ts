/* @layer renderer-app @kind logic */
import { formatDate } from '@drizztdourden08/tessera';
import type { CacheRow } from '../../../../../data/cache';

const unusedNote = (row: CacheRow, now: number): string => {
  const due = row.deleteAfter ? Date.parse(row.deleteAfter) : Number.NaN;
  if (!Number.isFinite(due) || due <= now) return 'unused, deleted at the next prune';
  return `unused, deleted after ${formatDate(due, 'date')}`;
};

export { unusedNote };
