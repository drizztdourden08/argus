/* @layer renderer-app @kind logic */
import { useCallback } from 'react';
import { removeWorktree } from '../../../../../data/worktrees';
import { failureText, isLeftover, leftoverNotice, removalInput, removalNotice } from '../model';
import type { WorktreeRow } from '../model';
import type { Guard, Notices, PanelControls, Removal } from '../worktrees.type';

const useRemoval = (boardKey: string, controls: PanelControls, guard: Guard, notices: Notices): Removal => {
  const { panel, patch, close } = controls;
  const { attempt } = guard;
  const confirm = useCallback(async (row: WorktreeRow) => {
    if (panel?.id !== row.id || panel.kind !== 'remove') return;
    const outcome = await attempt(() => removeWorktree(boardKey, removalInput(row, panel)));
    if (!outcome) return;
    if (!outcome.ok) return patch({ error: failureText(outcome.error, 'Not removed') });
    notices.show(isLeftover(row) ? leftoverNotice(row, outcome.value) : removalNotice(row, outcome.value));
    close();
  }, [boardKey, panel, patch, close, attempt, notices]);
  return { confirm, busy: guard.busy };
};

export { useRemoval };
