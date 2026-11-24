/* @layer renderer-app @kind logic */
import { useCallback } from 'react';
import type { AccountActions } from '../accounts.type';
import { COPY_STARTED, SYNC_PATH, UNDO_PATH, UNDO_STARTED } from '../compare.constants';
import { usePostAction } from './usePostAction';

const useAccountActions = (): AccountActions => {
  const { busy, post } = usePostAction();
  const copy = useCallback((from: string, to: string) => post(SYNC_PATH, { dryRun: false, from, to }, COPY_STARTED), [post]);
  const undo = useCallback(() => post(UNDO_PATH, undefined, UNDO_STARTED), [post]);
  return { busy, copy, undo };
};

export { useAccountActions };
