/* @layer renderer-app @kind logic */
import { toast } from '@drizztdourden08/tessera';
import { useCallback, useState } from 'react';
import { apiRequest, errorMessage } from '../../../../data/api';
import type { AccountActions } from '../accounts.type';
import { SYNC_PATH, SYNC_STARTED, UNDO_PATH, UNDO_STARTED } from '../compare.constants';

const useAccountActions = (): AccountActions => {
  const [busy, setBusy] = useState(false);

  const post = useCallback((path: string, json: object | undefined, done: string) => {
    setBusy(true);
    apiRequest(path, { method: 'POST', json })
      .then(
        () => toast({ variant: 'success', message: done }),
        (failure: unknown) => toast({ variant: 'danger', message: errorMessage(failure) }),
      )
      .finally(() => setBusy(false));
  }, []);

  const sync = useCallback(() => post(SYNC_PATH, { dryRun: false }, SYNC_STARTED), [post]);
  const undo = useCallback(() => post(UNDO_PATH, undefined, UNDO_STARTED), [post]);

  return { busy, sync, undo };
};

export { useAccountActions };
