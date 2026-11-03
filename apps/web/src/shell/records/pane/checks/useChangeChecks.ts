/* @layer renderer-app @kind logic */
import { useCallback } from 'react';
import { CHECK_POLL_MS, readChecks, useRemoteRead } from '../../../../data/remote';
import type { CheckRun, RemoteRead } from '../../../../data/remote';

const useChangeChecks = (boardKey: string, pr: string): RemoteRead<CheckRun[]> => {
  const load = useCallback(() => readChecks(boardKey, { pr }), [boardKey, pr]);
  return useRemoteRead(load, CHECK_POLL_MS);
};

export { useChangeChecks };
