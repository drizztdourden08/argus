/* @layer renderer-app @kind logic */
import type { Worktree } from '../../../../../data/worktrees';
import { commitsText } from './commits-text';

const lostItems = (row: Worktree): string[] => {
  const items: string[] = [];
  const commits = commitsText(row);
  const named = Boolean(row.branch) && !row.detached;
  if (row.dirty) items.push(`${row.dirty} uncommitted ${row.dirty === 1 ? 'file' : 'files'} (changed or new)`);
  if (commits && !named) items.push(`${commits} (on a detached HEAD that no branch keeps)`);
  if (named) items.push(`the branch ${row.branch}${commits ? ` with its ${commits}` : ''}, if you also delete it`);
  return items;
};

export { lostItems };
