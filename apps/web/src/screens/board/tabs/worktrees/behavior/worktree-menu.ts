/* @layer renderer-app @kind logic */
import type { MenuGroup } from '@drizztdourden08/tessera';
import { recordMenuGroups } from '../../../../../shell';
import { copyText } from '../../../shared';
import { canRemove, threadOf } from '../model';
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

const worktreeMenu = (row: WorktreeRow, input: WorktreeMenuInput): MenuGroup[] => {
  const deleting = row.status === 'deleting';
  const handlers = {
    open: input.open,
    remove: !deleting && canRemove(row) ? () => input.controls.toggle(row, 'remove') : undefined,
    copy: () => copyText(input.code),
  };
  return recordMenuGroups(WORKTREE_KIND, input.label, handlers, deleting ? [] : [askGroup(row, input)]);
};

export { worktreeMenu };
