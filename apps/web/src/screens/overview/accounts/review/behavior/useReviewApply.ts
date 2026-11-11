/* @layer renderer-app @kind logic */
import { toast } from '@drizztdourden08/tessera';
import { useCallback, useState } from 'react';
import { apiRequest } from '../../../../../data/api';
import type { ReviewEdit } from '../../../../../data/types';
import { applyProblem } from '../model';
import { APPLY_PATH, APPLY_STARTED } from '../review.constants';
import type { ApplyState, Decisions } from '../review.type';

const useReviewApply = (onApplied: () => void): ApplyState => {
  const [busy, setBusy] = useState(false);

  const apply = useCallback((decisions: Decisions, edits: readonly ReviewEdit[]) => {
    setBusy(true);
    apiRequest(APPLY_PATH, { method: 'POST', json: { decisions, edits, closeDesktop: true } })
      .then(
        () => {
          toast({ variant: 'success', message: APPLY_STARTED });
          onApplied();
        },
        (failure: unknown) => toast({ variant: 'danger', message: applyProblem(failure) }),
      )
      .finally(() => setBusy(false));
  }, [onApplied]);

  return { busy, apply };
};

export { useReviewApply };
