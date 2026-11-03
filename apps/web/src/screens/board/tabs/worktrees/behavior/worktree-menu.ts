/* @layer renderer-app @kind logic */
import type { MenuGroup } from '@drizztdourden08/tessera';
import { recordMenuGroups } from '../../../../../shell';
import { copyText } from '../../../shared';
import { canRemove, removeLook, threadOf } from '../model';
import type { WorktreeRow } from '../model';
import { ASK_BUTTONS, WORKTREE_KIND } from '../worktrees.constants';
import type { WorktreeMenuInput } from '../worktrees.type';

const askGroup = (row: WorktreeRow, input: WorktreeMenuInput): MenuGroup => {
  const thread = threadOf(row, input.meta) || 'the thread';
  return {
    id: 'ask',
    items: ASK_BUTTONS.map((ask) => ({ id: `ask-${ask.kind}`, label: ask.label, description: ask.tip(thread), onSelect: () => input.controls.toggle(row, ask.kind) })),
  };
};

const removeGroup = (row: WorktreeRow, input: WorktreeMenuInput): MenuGroup[] => {
  if (!canRemove(row)) return [];
  const look = removeLook(row);
  return [{ id: 'remove', items: [{ id: 'remove', label: look.label, description: look.tip, tone: look.danger ? 'danger' : undefined, onSelect: () => input.controls.toggle(row, 'remove') }] }];
};

const worktreeMenu = (row: WorktreeRow, input: WorktreeMenuInput): MenuGroup[] => {
  const handlers = { open: input.open, copy: () => copyText(input.code) };
  const middle = row.status === 'deleting' ? [] : [askGroup(row, input), ...removeGroup(row, input)];
  return recordMenuGroups(WORKTREE_KIND, input.label, handlers, middle);
};

export { worktreeMenu };
