/* @layer renderer-app @kind logic */
import { toast } from '@drizztdourden08/tessera';
import { useCallback, useState } from 'react';
import { apiRequest, errorMessage } from '../../../../../data/api';
import { ANYWAY_PATH, ANYWAY_TOAST, CANCEL_PATH, SWITCH_CANCELLED_TOAST, SWITCH_PATH, SWITCH_STARTED } from '../switch.constants';
import type { SwitchActions } from '../switch.type';

const useSwitchActions = (): SwitchActions => {
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

  const start = useCallback((to: string, handover: boolean) => post(SWITCH_PATH, { to, handover }, SWITCH_STARTED), [post]);
  const anyway = useCallback(() => post(ANYWAY_PATH, undefined, ANYWAY_TOAST), [post]);
  const cancel = useCallback(() => post(CANCEL_PATH, undefined, SWITCH_CANCELLED_TOAST), [post]);

  return { busy, start, anyway, cancel };
};

export { useSwitchActions };
