/* @layer renderer-app @kind logic */
import { useCallback, useState } from 'react';
import type { CollapseState } from '../review.type';

const useCollapsed = (): CollapseState => {
  const [collapsed, setCollapsed] = useState<ReadonlySet<string>>(new Set());

  const toggle = useCallback((rowId: string) => {
    setCollapsed((prev) => {
      const next = new Set(prev);
      if (!next.delete(rowId)) next.add(rowId);
      return next;
    });
  }, []);

  const setAll = useCallback((ids: ReadonlySet<string>) => setCollapsed(ids), []);

  return { collapsed, toggle, setAll };
};

export { useCollapsed };
