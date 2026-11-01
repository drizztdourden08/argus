/* @layer renderer-app @kind logic */
import type { Worktree } from '../../../../../data/worktrees';
import { STALENESS_STATUSES } from './model.constants';

// The staleness status (stale, merged, dirty and stale) only explains a worktree in review. "active" (touched in the
// last days) says nothing about use and is never shown: the hygiene state is the state.
const statusShown = (row: Worktree): boolean => {
  if (row.status === 'active') return false;
  const state = row.hygiene?.state;
  if (state === undefined || state === 'review') return true;
  return !STALENESS_STATUSES.includes(row.status);
};

export { statusShown };
