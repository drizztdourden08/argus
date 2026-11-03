/* @layer renderer-app @kind logic */
import { useCallback } from 'react';
import { readChecks, useRemoteRead } from '../../../../data/remote';
import type { CheckRun } from '../../../../data/remote';

const useCheckRun = (boardKey: string, id: string): { run: CheckRun | null; ready: boolean } => {
  const load = useCallback(() => readChecks(boardKey), [boardKey]);
  const { data, error } = useRemoteRead(load);
  return { run: data?.find((candidate) => candidate.id === id) ?? null, ready: data !== null || error !== '' };
};

export { useCheckRun };
