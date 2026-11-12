/* @layer renderer-app @kind logic */
import { toast } from '@drizztdourden08/tessera';
import { useCallback, useState } from 'react';
import { apiRequest, errorMessage } from '../../../../data/api';
import { QUIT_CANCEL_PATH } from '../compare.constants';

const useQuitCancel = (): { busy: boolean; run: () => void } => {
  const [busy, setBusy] = useState(false);
  const run = useCallback(() => {
    setBusy(true);
    apiRequest(QUIT_CANCEL_PATH, { method: 'POST' })
      .catch((failure: unknown) => toast({ variant: 'danger', message: errorMessage(failure) }))
      .finally(() => setBusy(false));
  }, []);
  return { busy, run };
};

export { useQuitCancel };
