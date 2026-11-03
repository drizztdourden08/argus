/* @layer renderer-app @kind logic */
import type { Worktree } from '../../../../../data/worktrees';
import { STALENESS_STATUSES } from './model.constants';

const statusShown = (row: Worktree): boolean => {
  if (row.status === 'active') return false;
  const state = row.hygiene?.state;
  if (state === undefined || state === 'review') return true;
  return !STALENESS_STATUSES.includes(row.status);
};

export { statusShown };
