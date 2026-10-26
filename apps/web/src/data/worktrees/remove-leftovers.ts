/* @layer renderer-app @kind logic */
import { apiRequest, scopePath } from '../api';
import { BATCH_PATH } from './worktrees.constants';
import type { BatchResult } from './worktrees.type';

const removeLeftovers = (key: string, olderThanDays: number): Promise<BatchResult> =>
  apiRequest<BatchResult>(scopePath(key, ...BATCH_PATH), { method: 'POST', json: { olderThanDays } });

export { removeLeftovers };
