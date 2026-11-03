/* @layer renderer-app @kind logic */
import type { RemovedWorktree } from '../../../../../data/worktrees';
import { folderOf } from './folder-of';

const removedTitle = (done: RemovedWorktree): string => {
  const folder = folderOf(done.path);
  const own = !done.branch || done.branch === folder || done.branch === `worktree-${folder}`;
  return own ? '' : (done.branch ?? '');
};

export { removedTitle };
