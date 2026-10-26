/* @layer renderer-app @kind logic */
import type { Worktree } from '../../../../../data/worktrees';

const nameOf = (row: Worktree): string => row.name ?? row.path;

export { nameOf };
