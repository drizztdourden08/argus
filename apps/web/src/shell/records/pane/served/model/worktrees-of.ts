/* @layer renderer-app @kind logic */
import type { WithId } from '../../../../../data/react';
import type { Worktree } from '../../../../../data/worktrees';

const worktreesOf = (docs: readonly WithId<Worktree>[], itemId: string): WithId<Worktree>[] =>
  docs.filter((row) => row.item === itemId).sort((a, b) => a.path.localeCompare(b.path));

export { worktreesOf };
