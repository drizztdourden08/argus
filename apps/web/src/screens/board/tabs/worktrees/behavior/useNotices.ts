/* @layer renderer-app @kind logic */
import { useCallback, useMemo, useState } from 'react';
import { toast } from '@drizztdourden08/tessera';
import type { Notice } from '../model';
import type { Notices } from '../worktrees.type';

const useNotices = (): Notices => {
  const [notice, setNotice] = useState<Notice | null>(null);
  const [dismissed, setDismissed] = useState<ReadonlySet<string>>(new Set());
  const show = useCallback((next: Notice) => {
    setNotice(next);
    toast({ variant: 'success', message: next.text });
  }, []);
  const clear = useCallback(() => setNotice(null), []);
  const dismissJobs = useCallback((ids: readonly string[]) => setDismissed((current) => new Set([...current, ...ids])), []);
  return useMemo(() => ({ notice, dismissed, show, clear, dismissJobs }), [notice, dismissed, show, clear, dismissJobs]);
};

export { useNotices };
