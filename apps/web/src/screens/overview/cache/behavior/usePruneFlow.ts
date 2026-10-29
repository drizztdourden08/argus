/* @layer renderer-app @kind logic */
import { toast } from '@drizztdourden08/tessera';
import { useCallback, useState } from 'react';
import { pruneCache } from '../../../../data/cache';
import type { PruneFlow } from '../cache.type';
import { cacheError, pruneSaid } from '../model';

const usePruneFlow = (reload: () => void): PruneFlow => {
  const [asking, setAsking] = useState(false);
  const [busy, setBusy] = useState(false);

  const confirm = useCallback(() => {
    if (busy) return;
    setBusy(true);
    pruneCache()
      .then(
        (result) => toast(pruneSaid(result)),
        (failure: unknown) => toast({ variant: 'danger', message: cacheError(failure, (detail) => `The prune failed: ${detail}.`) }),
      )
      .finally(() => {
        setBusy(false);
        setAsking(false);
        reload();
      });
  }, [busy, reload]);

  return { asking, busy, ask: () => setAsking(true), cancel: () => setAsking(false), confirm };
};

export { usePruneFlow };
