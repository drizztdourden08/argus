/* @layer renderer-app @kind logic */
import type { Worktree } from '../../../../../data/worktrees';
import { aboutSize } from './about-size';
import { isLowerBound } from './is-lower-bound';
import { sizeText } from './size-text';

const confirmSize = (row: Worktree): string => {
  const mb = row.leftover?.sizeMB;
  return isLowerBound(row) && mb !== null && mb !== undefined && mb >= 1 ? `at least ${sizeText(mb)}` : aboutSize(mb);
};

export { confirmSize };
