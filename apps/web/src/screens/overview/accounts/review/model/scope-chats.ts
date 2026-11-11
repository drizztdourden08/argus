/* @layer renderer-app @kind logic */
import type { ReviewRow } from '../../../../../data/types';
import type { BulkWant } from '../review.type';

const fits = (want: BulkWant, archived: boolean): boolean => want === 'any' || (want === 'archived') === archived;

const inScope = (row: ReviewRow, account: string, want: BulkWant): boolean => {
  const cell = row.cells[account];
  return row.kind === 'chat' && cell?.present === true && fits(want, cell.archived === true);
};

const scopeChats = (scope: readonly ReviewRow[], account: string, want: BulkWant): string[] => scope.filter((row) => inScope(row, account, want)).map((row) => row.id);

export { scopeChats };
