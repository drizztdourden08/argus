/* @layer renderer-app @kind logic */
import type { PostResult } from '../api';
import { ownerEntry, postEntry } from '../posts';
import type { WorktreeAskInput } from './worktrees.type';

const askWorktree = (key: string, input: WorktreeAskInput): Promise<PostResult> => {
  const { id, path, kind, thread, message } = input;
  const text = `[worktree ${kind}] ${path}:${message ? ` ${message}` : ''}`;
  return postEntry(key, ownerEntry({ kind: 'followup', thread, text, worktree: { id, path, kind, message } }));
};

export { askWorktree };
