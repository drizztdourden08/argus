/* @layer renderer-app @kind logic */
import type { ReviewRow } from '../../../../../data/types';
import type { BulkWant } from '../review.type';

const fits = (want: BulkWant, archived: boolean): boolean => want === 'any' || (want === 'archived') === archived;

const bulkIds = (rows: readonly ReviewRow[], account: string, ids: readonly string[], want: BulkWant): string[] => {
  const picked = new Set(ids);
  return rows.filter((row) => row.kind === 'chat' && picked.has(row.id) && fits(want, row.cells[account]?.archived === true)).map((row) => row.id);
};

export { bulkIds };
