/* @layer renderer-app @kind logic */
import type { LeftoverInput, RemoveInput } from '../../../../../data/worktrees';
import { isLeftover } from './is-leftover';
import type { Panel, WorktreeRow } from './model.type';

const removalInput = (row: WorktreeRow, panel: Panel): RemoveInput | LeftoverInput => {
  const { seen } = panel;
  if (isLeftover(row)) return { id: row.id, newestAt: seen.newestAt, confirmYoung: seen.young };
  return { id: row.id, branch: panel.branch && Boolean(row.branch), discard: seen.discard, head: seen.head, dirty: seen.dirty };
};

export { removalInput };
