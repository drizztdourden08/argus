/* @layer renderer-app @kind logic */
import { useCallback, useState } from 'react';
import { STORE_KEYS } from '../model';
import type { FocusState } from '../kanban.type';
import { readStored } from './read-stored';
import { useStoredFlag } from './useStoredFlag';
import { writeStored } from './write-stored';

const useFocusState = (): FocusState => {
  const [focus, setFocusValue] = useState(() => readStored(STORE_KEYS.focus) ?? '');
  const [showPaused, setShowPaused] = useStoredFlag(STORE_KEYS.showPaused);
  const setFocus = useCallback((next: string) => {
    writeStored(STORE_KEYS.focus, next);
    setFocusValue(next);
  }, []);
  return { focus, showPaused, setFocus, setShowPaused };
};

export { useFocusState };
