/* @layer renderer-app @kind logic */
import { kindOf } from '../../../../../data/registry';
import type { RemovedWorktree } from '../../../../../data/worktrees';
import { WORKTREE_KIND } from '../worktrees.constants';
import { folderOf } from './folder-of';

const removedLabel = (done: RemovedWorktree): string => `${kindOf(WORKTREE_KIND)?.code ?? 'WT'} ${folderOf(done.path)}`;

export { removedLabel };
