/* @layer renderer-app @kind logic */
import { useCallback, useState } from 'react';
import type { Selection, SelectionState } from '../review.type';

const useSelection = (): SelectionState => {
  const [selection, setSelection] = useState<Selection>({});

  const set = useCallback((account: string, ids: readonly string[], on: boolean) => {
    setSelection((prev) => {
      const next = new Set(prev[account] ?? []);
      for (const id of ids) {
        if (on) next.add(id);
        else next.delete(id);
      }
      return { ...prev, [account]: [...next] };
    });
  }, []);

  const clear = useCallback((account: string) => setSelection((prev) => ({ ...prev, [account]: [] })), []);

  return { selection, set, clear };
};

export { useSelection };
