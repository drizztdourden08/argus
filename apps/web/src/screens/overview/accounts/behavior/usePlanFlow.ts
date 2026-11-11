/* @layer renderer-app @kind logic */
import { useCallback, useState } from 'react';
import { apiRequest, errorMessage } from '../../../../data/api';
import { SYNC_PATH } from '../compare.constants';
import type { PlanFlow } from '../compare.type';
import { summaryText } from '../model';

const usePlanFlow = (): PlanFlow => {
  const [busy, setBusy] = useState(false);
  const [note, setNote] = useState<string | null>(null);

  const run = useCallback(() => {
    setBusy(true);
    apiRequest<unknown>(SYNC_PATH, { method: 'POST', json: { dryRun: true } })
      .then(
        (result) => setNote(summaryText(result)),
        (failure: unknown) => setNote(errorMessage(failure)),
      )
      .finally(() => setBusy(false));
  }, []);

  return { busy, note, run };
};

export { usePlanFlow };
