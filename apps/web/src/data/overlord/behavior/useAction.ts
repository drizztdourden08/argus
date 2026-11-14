/* @layer renderer-app @kind logic */
import { useState } from 'react';
import { errorMessage } from '../../api';
import type { ActionState } from '../overlord.type';

const useAction = (onDone: () => void): ActionState => {
  const [busy, setBusy] = useState(false);
  const [problem, setProblem] = useState<string | null>(null);

  const run = async (action: () => Promise<unknown>): Promise<boolean> => {
    setBusy(true);
    setProblem(null);
    try {
      await action();
      onDone();
      return true;
    } catch (failure) {
      setProblem(errorMessage(failure));
      return false;
    } finally {
      setBusy(false);
    }
  };

  return { busy, problem, run };
};

export { useAction };
