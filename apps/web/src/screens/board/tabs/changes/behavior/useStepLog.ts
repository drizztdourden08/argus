/* @layer renderer-app @kind logic */
import { useCallback } from 'react';
import { readCheckLog, useRemoteRead } from '../../../../../data/remote';
import type { RemoteRead } from '../../../../../data/remote';
import type { LogTarget } from '../changes.type';

const useStepLog = (boardKey: string, target: LogTarget): RemoteRead<string> => {
  const { runId, index } = target;
  const load = useCallback(() => readCheckLog(boardKey, runId, index), [boardKey, runId, index]);
  return useRemoteRead(load);
};

export { useStepLog };
