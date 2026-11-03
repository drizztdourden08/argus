/* @layer renderer-app @kind logic */
import type { WorktreeRequest, WorktreesMeta } from '../../../../../data/worktrees';

const requestsOf = (meta: WorktreesMeta | null): WorktreeRequest[] => meta?.requests ?? [];

export { requestsOf };
