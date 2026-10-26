/* @layer renderer-app @kind logic */
import type { Worktree, WorktreesMeta } from '../../../../../data/worktrees';

const threadOf = (row: Worktree, meta: WorktreesMeta | null): string => {
  if (row.thread) return row.thread;
  return meta?.rules?.find((rule) => rule.id === row.rule)?.thread ?? '';
};

export { threadOf };
