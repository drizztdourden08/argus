/* @layer renderer-app @kind logic */
import { useCallback } from 'react';
import type { AccountActions } from '../accounts.type';
import { COPY_STARTED, SYNC_PATH, SYNC_STARTED, UNDO_PATH, UNDO_STARTED } from '../compare.constants';
import { usePostAction } from './usePostAction';

const useAccountActions = (): AccountActions => {
  const { busy, post } = usePostAction();
  const merge = useCallback(() => post(SYNC_PATH, { dryRun: false }, SYNC_STARTED), [post]);
  const copy = useCallback((from: string, to: string) => post(SYNC_PATH, { dryRun: false, from, to }, COPY_STARTED), [post]);
  const undo = useCallback(() => post(UNDO_PATH, undefined, UNDO_STARTED), [post]);
  return { busy, merge, copy, undo };
};

export { useAccountActions };
