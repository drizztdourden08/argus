/* @layer renderer-app @kind logic */
import { useCallback } from 'react';
import { NEXT_CLEARED, NEXT_PATH, NEXT_SAVED } from '../next.constants';
import type { NextActions } from '../next.type';
import { usePostAction } from './usePostAction';

const useNextActions = (reload: () => void): NextActions => {
  const { busy, post } = usePostAction(reload);
  const choose = useCallback((from: string | null, to: string | null) => post(NEXT_PATH, { from, to }, NEXT_SAVED), [post]);
  const clear = useCallback(() => post(NEXT_PATH, { clear: true }, NEXT_CLEARED), [post]);
  return { busy, choose, clear };
};

export { useNextActions };
