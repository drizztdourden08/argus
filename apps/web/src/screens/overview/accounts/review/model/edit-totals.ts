/* @layer renderer-app @kind logic */
import type { ReviewEdit } from '../../../../../data/types';
import { EDIT_LABEL } from '../review.constants';
import type { TotalLine } from '../review.type';

const editTotals = (edits: readonly ReviewEdit[]): TotalLine[] => {
  const counts = new Map<string, number>();
  for (const edit of edits) counts.set(EDIT_LABEL[edit.op], (counts.get(EDIT_LABEL[edit.op]) ?? 0) + 1);
  return [...counts].map(([label, count]) => ({ label, count }));
};

export { editTotals };
