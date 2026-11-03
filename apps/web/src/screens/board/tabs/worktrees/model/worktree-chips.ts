/* @layer renderer-app @kind logic */
import { cellSize } from './cell-size';
import { hygieneTag } from './hygiene-tag';
import { isLeftover } from './is-leftover';
import { leftoverFlags } from './leftover-flags';
import { STATUS_LABEL } from './model.constants';
import type { WorktreeChip, WorktreeRow } from './model.type';
import { statusShown } from './status-shown';
import { statusTip } from './status-tip';

const stateChips = (row: WorktreeRow): WorktreeChip[] => {
  const chips: WorktreeChip[] = [];
  if (statusShown(row)) chips.push({ id: 'status', text: STATUS_LABEL[row.status] ?? row.status, title: statusTip(row) });
  if (row.hygiene) {
    const tag = hygieneTag(row.hygiene);
    chips.push({ id: 'state', text: tag.text, title: tag.tip });
  }
  return chips;
};

const countChips = (row: WorktreeRow): WorktreeChip[] => {
  const chips: WorktreeChip[] = [];
  if (isLeftover(row) && row.leftover) chips.push({ id: 'size', text: cellSize(row), title: 'Size of the folder' });
  if (row.dirty) chips.push({ id: 'dirty', text: `dirty ${row.dirty}`, title: 'Files with uncommitted changes' });
  if (row.ahead) chips.push({ id: 'ahead', text: `ahead ${row.ahead}`, title: 'Commits not yet on the default branch' });
  return chips;
};

const flagChips = (row: WorktreeRow): WorktreeChip[] => {
  const chips: WorktreeChip[] = [];
  if (!row.item && row.unlinked) chips.push({ id: 'no-item', text: 'no item', title: 'No item an hour after Argus first saw it: link it (coord worktree link) or remove it.' });
  if (row.request) chips.push({ id: 'request', text: row.request.answer ? 'answered' : 'asked', title: 'A request to the thread about this worktree' });
  if (row.locked) chips.push({ id: 'locked', text: 'locked', title: `Locked: ${String(row.locked)}` });
  if (row.detached) chips.push({ id: 'detached', text: 'detached', title: 'HEAD is detached' });
  for (const flag of leftoverFlags(row)) chips.push({ id: `flag-${flag.text}`, text: flag.text, title: flag.tip });
  return chips;
};

const worktreeChips = (row: WorktreeRow): WorktreeChip[] => [...stateChips(row), ...countChips(row), ...flagChips(row)];

export { worktreeChips };
