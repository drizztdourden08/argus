/* @layer renderer-app @kind logic */
import { useCallback, useRef, useState } from 'react';
import type { Attempt, Guard } from '../worktrees.type';

const useAttempt = (): Guard => {
  const flying = useRef(false);
  const [busy, setBusy] = useState(false);
  const attempt = useCallback(async <T,>(task: () => Promise<T>): Promise<Attempt<T> | null> => {
    if (flying.current) return null;
    flying.current = true;
    setBusy(true);
    try {
      return { ok: true, value: await task() };
    } catch (error) {
      return { ok: false, error };
    } finally {
      flying.current = false;
      setBusy(false);
    }
  }, []);
  return { busy, attempt };
};

export { useAttempt };
