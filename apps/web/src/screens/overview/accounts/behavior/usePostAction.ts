/* @layer renderer-app @kind logic */
import { toast } from '@drizztdourden08/tessera';
import { useCallback, useState } from 'react';
import { apiRequest, errorMessage } from '../../../../data/api';
import type { PostAction } from '../accounts.type';

const usePostAction = (): PostAction => {
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

  return { busy, post };
};

export { usePostAction };
