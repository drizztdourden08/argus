/* @layer renderer-app @kind logic */
import { kindOf } from '../../../../../data/registry';
import { WORKTREE_KIND } from '../worktrees.constants';
import { folderOf } from './folder-of';
import type { WorktreeRow } from './model.type';

const worktreeLabel = (row: WorktreeRow): string => `${kindOf(WORKTREE_KIND)?.code ?? 'WT'} ${row.name ?? folderOf(row.path)}`;

export { worktreeLabel };
