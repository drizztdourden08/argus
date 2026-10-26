/* @layer renderer-app @kind logic */
import { toast } from '@drizztdourden08/tessera';
import { useCallback, useState } from 'react';
import { writeErrorText } from '../../../../data/api';
import type { RunAction } from '../run-play.type';

const useRunAction = (): RunAction => {
  const [busy, setBusy] = useState(false);
  const act = useCallback(async (action: () => Promise<string>) => {
    setBusy(true);
    try {
      toast({ variant: 'success', message: await action() });
    } catch (error) {
      toast({ variant: 'danger', message: writeErrorText(error) });
    } finally {
      setBusy(false);
    }
  }, []);
  return { busy, act };
};

export { useRunAction };
