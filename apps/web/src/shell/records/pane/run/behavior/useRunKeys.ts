/* @layer renderer-app @kind logic */
import { useEffect } from 'react';
import { handleKey } from '../model/handle-key';
import type { RunKeyHandlers } from '../run.type';

const useRunKeys = (handlers: RunKeyHandlers): void => {
  const { onRecord, onStep } = handlers;
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => handleKey(event, { onRecord, onStep });
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [onRecord, onStep]);
};

export { useRunKeys };
