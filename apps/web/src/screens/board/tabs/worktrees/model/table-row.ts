/* @layer renderer-app @kind logic */
import { isLeftover } from './is-leftover';
import type { TableRow, WorktreeRow } from './model.type';

const touchedOf = (row: WorktreeRow): string => (isLeftover(row) ? row.leftover?.newestAt : row.lastTouchedAt) ?? '';

const requestOf = (row: WorktreeRow): string => {
  if (!row.request) return '';
  return row.request.answer ? 'answered' : 'asked';
};

const tableRow = (row: WorktreeRow): TableRow => ({
  id: row.id,
  path: row.path,
  status: row.status,
  branch: row.branch ?? '',
  item: row.item ?? '',
  touched: touchedOf(row),
  ahead: row.ahead ?? 0,
  dirty: row.dirty ?? 0,
  subject: row.subject ?? '',
  request: requestOf(row),
  actions: '',
  source: row,
});

export { tableRow };
