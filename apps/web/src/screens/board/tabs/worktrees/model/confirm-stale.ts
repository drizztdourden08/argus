/* @layer renderer-app @kind logic */
import { canRemove } from './can-remove';
import { isLeftover } from './is-leftover';
import { isSafe } from './is-safe';
import type { Panel, WorktreeRow } from './model.type';

const leftoverMoved = (row: WorktreeRow, panel: Panel): boolean => panel.seen.newestAt !== (row.leftover?.newestAt ?? null);

const worktreeMoved = (row: WorktreeRow, panel: Panel): boolean => {
  const { seen } = panel;
  return seen.head !== (row.head ?? null) || seen.dirty !== (row.dirty ?? 0) || seen.discard === isSafe(row);
};

const confirmStale = (row: WorktreeRow, panel: Panel): boolean => {
  if (!canRemove(row)) return true;
  return isLeftover(row) ? leftoverMoved(row, panel) : worktreeMoved(row, panel);
};

export { confirmStale };
