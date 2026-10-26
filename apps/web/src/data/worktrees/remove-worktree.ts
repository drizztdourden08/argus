/* @layer renderer-app @kind logic */
import { apiRequest, scopePath } from '../api';
import { REMOVE_PATH } from './worktrees.constants';
import type { LeftoverInput, RemoveInput, RemoveResult } from './worktrees.type';

const removeWorktree = (key: string, input: RemoveInput | LeftoverInput): Promise<RemoveResult> =>
  apiRequest<RemoveResult>(scopePath(key, ...REMOVE_PATH), { method: 'POST', json: input });

export { removeWorktree };
