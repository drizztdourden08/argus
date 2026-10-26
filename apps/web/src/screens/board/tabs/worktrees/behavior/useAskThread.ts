/* @layer renderer-app @kind logic */
import { toast } from '@drizztdourden08/tessera';
import { useCallback } from 'react';
import { askWorktree } from '../../../../../data/worktrees';
import type { WorktreesMeta } from '../../../../../data/worktrees';
import { askProblem, entryProblem, failureText, nameOf, threadOf } from '../model';
import type { WorktreeRow } from '../model';
import type { AskThread, Guard, PanelControls } from '../worktrees.type';

const useAskThread = (boardKey: string, meta: WorktreesMeta | null, controls: PanelControls, guard: Guard): AskThread => {
  const { panel, patch, close } = controls;
  const { attempt } = guard;
  const send = useCallback(async (row: WorktreeRow) => {
    if (panel?.id !== row.id || panel.kind === 'remove') return;
    const thread = threadOf(row, meta);
    const message = panel.text.trim();
    const problem = askProblem(thread, message);
    if (problem) return patch({ error: problem });
    const kind = panel.kind;
    const outcome = await attempt(() => askWorktree(boardKey, { id: row.id, path: row.path, kind, thread, message }));
    if (!outcome) return;
    const refused = outcome.ok ? entryProblem(outcome.value) : failureText(outcome.error, 'Write refused');
    if (refused) return patch({ error: refused });
    toast({ variant: 'success', message: `Asked ${thread} to ${kind} ${nameOf(row)}.` });
    close();
  }, [boardKey, meta, panel, patch, close, attempt]);
  return { send, busy: guard.busy };
};

export { useAskThread };
