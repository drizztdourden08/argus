/* @layer renderer-app @kind logic */
import type { Worktree } from '../../../../../data/worktrees';
import { isNeeding } from './is-needing';

const compareRows = (a: Worktree, b: Worktree): number =>
  Number(isNeeding(b)) - Number(isNeeding(a))
  || Number(b.status === 'missing') - Number(a.status === 'missing')
  || (b.ageDays ?? 0) - (a.ageDays ?? 0)
  || a.path.localeCompare(b.path);

export { compareRows };
