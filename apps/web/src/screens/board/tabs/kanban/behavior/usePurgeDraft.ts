/* @layer renderer-app @kind logic */
import { useCallback, useState } from 'react';
import { DEFAULT_PURGE } from '../kanban.constants';
import type { PurgeDraft, PurgeScope } from '../trash';

const usePurgeDraft = (thread: string, scope: PurgeScope): [PurgeDraft, (patch: Partial<PurgeDraft>) => void] => {
  const [draft, setDraft] = useState<PurgeDraft>({ ...DEFAULT_PURGE, thread, scope });
  const patch = useCallback((next: Partial<PurgeDraft>) => setDraft((now) => ({ ...now, ...next })), []);
  return [draft, patch];
};

export { usePurgeDraft };
