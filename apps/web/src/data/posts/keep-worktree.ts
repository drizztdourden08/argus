/* @layer renderer-app @kind logic */
import type { PostResult } from '../api';
import { postOwner } from './post-owner';

const keepWorktree = (key: string, id: string, note: string, days: number): Promise<PostResult> =>
  postOwner(key, { kind: 'worktree-keep', target: { c: 'worktrees', id }, set: { note, days } });

export { keepWorktree };
