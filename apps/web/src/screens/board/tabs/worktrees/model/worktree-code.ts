/* @layer renderer-app @kind logic */
import { kindOf } from '../../../../../data/registry';
import { shownCode } from '../../../../../shell/records';
import { WORKTREE_KIND } from '../worktrees.constants';
import type { WorktreeRow } from './model.type';

const worktreeCode = (row: WorktreeRow): string => {
  const kind = kindOf(WORKTREE_KIND);
  return kind ? shownCode(kind, row.id) : `WT-${row.id}`;
};

export { worktreeCode };
