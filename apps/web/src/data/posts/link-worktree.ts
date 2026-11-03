/* @layer renderer-app @kind logic */
import type { PostResult } from '../api';
import { postOwner } from './post-owner';

const linkWorktree = (key: string, id: string, item: string | null): Promise<PostResult> =>
  postOwner(key, { kind: 'worktree-link', target: { c: 'worktrees', id }, set: { item } });

export { linkWorktree };
