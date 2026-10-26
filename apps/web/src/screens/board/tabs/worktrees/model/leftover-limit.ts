/* @layer renderer-app @kind logic */
import type { WorktreesMeta } from '../../../../../data/worktrees';
import { DEFAULT_LIMIT_DAYS } from './model.constants';

const leftoverLimit = (meta: WorktreesMeta | null): number => {
  const days = meta?.leftoverOld?.days;
  return days ?? DEFAULT_LIMIT_DAYS;
};

export { leftoverLimit };
