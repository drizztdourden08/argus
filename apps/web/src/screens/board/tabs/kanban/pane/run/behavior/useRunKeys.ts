/* @layer renderer-app @kind logic */
import { useEffect } from 'react';
import { TYPING_SELECTOR } from '../../../kanban.constants';
import { keyStatus } from '../model/key-status';
import { keyStep } from '../model/key-step';
import { STRIP_SELECTOR } from '../run.constants';
import type { RunKeyHandlers } from '../run.type';

const useRunKeys = (handlers: RunKeyHandlers): void => {
  const { onRecord, onStep } = handlers;
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      const target = event.target instanceof Element ? event.target : null;
      if (target?.closest(TYPING_SELECTOR) || event.ctrlKey || event.metaKey || event.altKey || event.repeat) return;
      const by = keyStep(event.key);
      const status = keyStatus(event.key);
      if (by !== null && !target?.closest(STRIP_SELECTOR)) onStep(by);
      else if (status) onRecord(status);
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [onRecord, onStep]);
};

export { useRunKeys };
