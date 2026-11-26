/* @layer renderer-app @kind logic */
import { useCallback } from 'react';
import { NEXT_CLEARED, NEXT_PATH, NEXT_SAVED } from '../next.constants';
import type { NextActions, SlotPair } from '../next.type';
import { usePostAction } from './usePostAction';

const useNextActions = (reload: () => void): NextActions => {
  const { busy, post } = usePostAction(reload);
  const save = useCallback((pair: SlotPair) => {
    if (pair.left === null && pair.right === null) post(NEXT_PATH, { clear: true }, NEXT_CLEARED);
    else post(NEXT_PATH, { from: pair.left, to: pair.right }, NEXT_SAVED);
  }, [post]);
  return { busy, save };
};

export { useNextActions };
