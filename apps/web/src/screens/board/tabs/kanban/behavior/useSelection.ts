/* @layer renderer-app @kind logic */
import { useCallback, useState } from 'react';
import type { Selection } from '../kanban.type';

const useSelection = (): Selection => {
  const [picked, setPicked] = useState<ReadonlySet<string>>(new Set());
  const has = useCallback((id: string) => picked.has(id), [picked]);
  const toggle = useCallback((id: string) => {
    setPicked((now) => {
      const next = new Set(now);
      if (!next.delete(id)) next.add(id);
      return next;
    });
  }, []);
  const clear = useCallback(() => setPicked(new Set()), []);
  return { has, toggle, clear };
};

export { useSelection };
