/* @layer renderer-app @kind logic */
import { toast } from '@drizztdourden08/tessera';
import { useCallback, useState } from 'react';
import { writeErrorText } from '../../../../../data/api';
import { cancelDep } from '../../../../../data/deps';
import type { CancelDep } from '../kanban.type';

const useCancelDep = (dep: string): CancelDep => {
  const [busy, setBusy] = useState(false);
  const cancel = useCallback(async () => {
    setBusy(true);
    try {
      toast({ variant: 'success', message: (await cancelDep(dep)).text });
    } catch (error) {
      toast({ variant: 'danger', message: writeErrorText(error) });
    } finally {
      setBusy(false);
    }
  }, [dep]);
  return { busy, cancel };
};

export { useCancelDep };
