/* @layer renderer-app @kind logic */
import { useCallback } from 'react';
import { CHECK_POLL_MS, readChecks, useRemoteRead } from '../../../../../data/remote';
import type { CheckRun, RemoteRead } from '../../../../../data/remote';
import type { PullRequestDoc } from '../changes.type';

const usePrChecks = (boardKey: string, pr: PullRequestDoc): RemoteRead<CheckRun[]> => {
  const known = pr.checks?.join(',') ?? '';
  const load = useCallback(() => readChecks(boardKey, { pr: pr.id }), [boardKey, pr.id, known]);
  return useRemoteRead(load, pr.status === 'open' ? CHECK_POLL_MS : 0);
};

export { usePrChecks };
