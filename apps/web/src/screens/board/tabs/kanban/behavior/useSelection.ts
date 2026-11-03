/* @layer renderer-app @kind logic */
import { useCallback, useRef, useState } from 'react';
import { rangeOf } from '../model';
import type { Selection } from '../kanban.type';

const useSelection = (): Selection => {
  const [picked, setPicked] = useState<ReadonlySet<string>>(new Set());
  const [active, setOn] = useState(false);
  const anchor = useRef<string | null>(null);
  const shifted = useRef(false);
  const has = useCallback((id: string) => picked.has(id), [picked]);
  const mark = useCallback((shift: boolean) => {
    shifted.current = shift;
  }, []);
  const clear = useCallback(() => {
    anchor.current = null;
    setPicked(new Set());
  }, []);
  const setActive = useCallback(
    (on: boolean) => {
      setOn(on);
      if (!on) clear();
    },
    [clear],
  );
  const toggle = useCallback(
    (id: string, order: readonly string[]) => {
      const on = !picked.has(id);
      const span = rangeOf(order, shifted.current ? anchor.current : null, id);
      shifted.current = false;
      anchor.current = id;
      setOn(true);
      setPicked((now) => {
        const next = new Set(now);
        for (const key of span) {
          if (on) next.add(key);
          else next.delete(key);
        }
        return next;
      });
    },
    [picked],
  );
  return { active, setActive, has, toggle, mark, clear };
};

export { useSelection };
