/* @layer renderer-app @kind logic */
import type { Worktree } from '../../../../../data/worktrees';
import { isLowerBound } from './is-lower-bound';
import { sizeText } from './size-text';

const cellSize = (row: Worktree): string => {
  const facts = row.leftover;
  if (!facts) return '';
  if (facts.pending) return 'measuring';
  return `${isLowerBound(row) ? 'at least ' : ''}${sizeText(facts.sizeMB)}`;
};

export { cellSize };
