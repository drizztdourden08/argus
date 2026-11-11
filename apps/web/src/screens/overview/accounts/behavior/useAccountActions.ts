/* @layer renderer-app @kind logic */
import { toast } from '@drizztdourden08/tessera';
import { useCallback, useState } from 'react';
import { apiRequest, errorMessage } from '../../../../data/api';
import type { AccountActions } from '../accounts.type';

const useAccountActions = (): AccountActions => {
  const [busy, setBusy] = useState(false);

  const post = useCallback((path: string, json: object, done: string) => {
    setBusy(true);
    apiRequest(path, { method: 'POST', json })
      .then(
        () => toast({ variant: 'success', message: done }),
        (failure: unknown) => toast({ variant: 'danger', message: errorMessage(failure) }),
      )
      .finally(() => setBusy(false));
  }, []);

  const switchTo = useCallback((key: string) => post('/api/accounts/switch', { to: key, handover: true }, `Switching Desktop to ${key}.`), [post]);
  const sync = useCallback(() => post('/api/accounts/sync', { dryRun: false }, 'Sidebar sync started.'), [post]);

  return { busy, switchTo, sync };
};

export { useAccountActions };
