/* @layer renderer-app @kind logic */
import { STORE_KEYS } from '../model';
import type { FocusState } from '../kanban.type';
import { useStored, useStoredFlag } from '../../../shared';

const useFocusState = (): FocusState => {
  const [focus, setFocus] = useStored(STORE_KEYS.focus);
  const [showPaused, setShowPaused] = useStoredFlag(STORE_KEYS.showPaused);
  return { focus, showPaused, setFocus, setShowPaused };
};

export { useFocusState };
