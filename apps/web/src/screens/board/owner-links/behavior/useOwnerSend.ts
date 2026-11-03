/* @layer renderer-app @kind logic */
import { toast } from '@drizztdourden08/tessera';
import { useCallback, useState } from 'react';
import { writeErrorText } from '../../../../data/api';
import type { PostResult } from '../../../../data/api';
import { postVerdict } from '../../tabs/tests/model';
import type { OwnerSend } from '../owner-links.type';

const useOwnerSend = (onClose: () => void): OwnerSend => {
  const [busy, setBusy] = useState(false);
  const [problem, setProblem] = useState<string | null>(null);
  const send = useCallback(async (post: () => Promise<PostResult>, done: string) => {
    setBusy(true);
    try {
      const verdict = postVerdict(await post());
      setProblem(verdict);
      if (!verdict) {
        toast({ variant: 'success', message: done });
        onClose();
      }
    } catch (error) {
      setProblem(writeErrorText(error));
    } finally {
      setBusy(false);
    }
  }, [onClose]);
  return { busy, problem, send };
};

export { useOwnerSend };
