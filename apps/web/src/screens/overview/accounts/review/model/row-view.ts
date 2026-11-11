/* @layer renderer-app @kind logic */
import type { ReviewRow } from '../../../../../data/types';
import type { TreeEnv } from '../review-tree-props.type';
import type { RowView } from '../review.type';
import { archivedNote } from './archived-note';
import { cellNote } from './cell-note';
import { choiceOf } from './choice-of';
import { scopeChats } from './scope-chats';
import { scopeOf } from './scope-of';

const rowView = (row: ReviewRow, account: string, env: TreeEnv): RowView => {
  const cell = row.cells[account] ?? { present: false };
  const present = cell.present === true;
  const archived = present ? archivedNote(cell, env.now) : null;
  const note = cellNote(choiceOf(row, env.decisions), account, cell) ?? (present ? null : 'not created');
  const count = present && row.kind === 'folder' ? scopeChats(scopeOf(row, env.scopes), account, 'any').length : null;
  return { present, kids: (env.scopes.get(row.id)?.length ?? 0) > 0, collapsed: env.collapsed.has(row.id), archived, note, count };
};

export { rowView };
