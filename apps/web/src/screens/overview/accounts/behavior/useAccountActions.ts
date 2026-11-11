/* @layer renderer-app @kind logic */
import { toast } from '@drizztdourden08/tessera';
import { useCallback, useState } from 'react';
import { apiRequest, errorMessage } from '../../../../data/api';
import type { AccountActions } from '../accounts.type';
import { SYNC_PATH, UNDO_PATH } from '../compare.constants';

const useAccountActions = (onSync: () => void): AccountActions => {
  const [busy, setBusy] = useState(false);

  const post = useCallback((path: string, json: object | undefined, done: string, after?: () => void) => {
    setBusy(true);
    apiRequest(path, { method: 'POST', json })
      .then(
        () => {
          toast({ variant: 'success', message: done });
          after?.();
        },
        (failure: unknown) => toast({ variant: 'danger', message: errorMessage(failure) }),
      )
      .finally(() => setBusy(false));
  }, []);

  const switchTo = useCallback((key: string) => post('/api/accounts/switch', { to: key, handover: true }, `Switching Desktop to ${key}.`), [post]);
  const sync = useCallback(() => post(SYNC_PATH, { dryRun: false }, 'Sidebar sync started.', onSync), [post, onSync]);
  const undo = useCallback(() => post(UNDO_PATH, undefined, 'Undoing the last sync.'), [post]);

  return { busy, switchTo, sync, undo };
};

export { useAccountActions };
