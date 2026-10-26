/* @layer renderer-app @kind logic */
import type { WorktreesMeta } from '../../../../../data/worktrees';
import { isSafe } from './is-safe';
import { isYoung } from './is-young';
import { leftoverLimit } from './leftover-limit';
import type { Seen, WorktreeRow } from './model.type';

const seenOf = (row: WorktreeRow, meta: WorktreesMeta | null): Seen => ({
  head: row.head ?? null,
  dirty: row.dirty ?? 0,
  newestAt: row.leftover?.newestAt ?? null,
  discard: !isSafe(row),
  young: isYoung(row, leftoverLimit(meta)),
});

export { seenOf };
