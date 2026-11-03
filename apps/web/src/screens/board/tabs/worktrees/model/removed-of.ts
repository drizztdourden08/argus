/* @layer renderer-app @kind logic */
import type { RemovedWorktree, WorktreesMeta } from '../../../../../data/worktrees';

const removedOf = (meta: WorktreesMeta | null): RemovedWorktree[] => meta?.hygiene?.removed ?? [];

export { removedOf };
