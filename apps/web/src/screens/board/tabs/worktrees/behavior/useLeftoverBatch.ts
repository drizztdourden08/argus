/* @layer renderer-app @kind logic */
import { useCallback, useMemo, useState } from 'react';
import { removeLeftovers } from '../../../../../data/worktrees';
import type { WorktreesMeta } from '../../../../../data/worktrees';
import { batchNotice, failureText } from '../model';
import type { BatchControls, BatchState, Guard, Notices } from '../worktrees.type';

const useLeftoverBatch = (boardKey: string, meta: WorktreesMeta | null, guard: Guard, notices: Notices): BatchControls => {
  const [state, setState] = useState<BatchState | null>(null);
  const old = meta?.leftoverOld;
  const { attempt } = guard;
  const toggle = useCallback(() => setState((current) => (current ? null : { error: null })), []);
  const cancel = useCallback(() => setState(null), []);
  const confirm = useCallback(async () => {
    if (!old || old.count === 0) return;
    const outcome = await attempt(() => removeLeftovers(boardKey, old.days));
    if (!outcome) return;
    if (!outcome.ok) return setState({ error: failureText(outcome.error, 'Not queued') });
    notices.show(batchNotice(outcome.value));
    setState(null);
  }, [boardKey, old, attempt, notices]);
  return useMemo(() => ({ state, busy: guard.busy, toggle, cancel, confirm }), [state, guard.busy, toggle, cancel, confirm]);
};

export { useLeftoverBatch };
