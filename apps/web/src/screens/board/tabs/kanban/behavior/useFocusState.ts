/* @layer renderer-app @kind logic */
import { STORE_KEYS } from '../model';
import type { FocusState } from '../kanban.type';
import { focusKey, useStored, useStoredFlag } from '../../../shared';

const useFocusState = (boardKey: string): FocusState => {
  const [focus, setFocus] = useStored(focusKey(boardKey));
  const [showPaused, setShowPaused] = useStoredFlag(STORE_KEYS.showPaused);
  return { focus, showPaused, setFocus, setShowPaused };
};

export { useFocusState };
