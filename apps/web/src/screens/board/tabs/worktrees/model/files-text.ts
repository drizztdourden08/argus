/* @layer renderer-app @kind logic */
import type { Worktree } from '../../../../../data/worktrees';

const filesText = (row: Worktree): string => {
  const files = row.leftover?.files;
  if (files === null || files === undefined) return '';
  return `${files} ${files === 1 ? 'file' : 'files'}`;
};

export { filesText };
