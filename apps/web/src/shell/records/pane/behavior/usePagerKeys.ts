/* @layer renderer-app @kind logic */
import { useEffect } from 'react';
import { TYPING_SELECTOR } from '../../../../screens/board/tabs/kanban/kanban.constants';

const usePagerKeys = (previous: string | null, next: string | null, onStep: (id: string) => void): void => {
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      const target = event.target instanceof Element ? event.target : null;
      if (target?.closest(TYPING_SELECTOR) || event.ctrlKey || event.metaKey || event.altKey) return;
      const steps: Record<string, string | null> = { '[': previous, ']': next };
      const to = steps[event.key];
      if (to) onStep(to);
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [previous, next, onStep]);
};

export { usePagerKeys };
