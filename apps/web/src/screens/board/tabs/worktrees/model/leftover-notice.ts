/* @layer renderer-app @kind logic */
import type { RemoveResult } from '../../../../../data/worktrees';
import type { Notice, WorktreeRow } from './model.type';
import { nameOf } from './name-of';

const leftoverNotice = (row: WorktreeRow, result: RemoveResult): Notice => ({
  text: `Deleting ${nameOf(row)} in the background (job ${result.jobId}); the row goes when the folder is gone.`,
  backup: null,
  restore: null,
});

export { leftoverNotice };
