/* @layer renderer-app @kind logic */
import type { Worktree } from '../../../../../data/worktrees';
import { STATUS_TIP } from './model.constants';

const keptTail = (row: Worktree): string => {
  const kept = row.kept;
  if (row.status !== 'kept' || !kept) return '';
  const note = kept.note ? `: ${kept.note}` : '';
  const by = kept.by ? ` (${kept.by})` : '';
  return ` Until ${(kept.until ?? '').slice(0, 10)}${note}${by}.`;
};

const statusTip = (row: Worktree): string => `${STATUS_TIP[row.status] ?? row.status}${keptTail(row)}${row.error ? ` ${row.error}` : ''}`;

export { statusTip };
