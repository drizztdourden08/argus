/* @layer renderer-app @kind logic */
import { useCallback } from 'react';
import { CHECK_POLL_MS, readChecks, useRemoteRead } from '../../../../../data/remote';
import type { PullRequestDoc } from '../changes.type';
import { failingPrIds } from '../model/failing-prs';
import { NO_FAILING } from './pr-failing.constants';

const usePrFailing = (boardKey: string, prs: readonly PullRequestDoc[]): ReadonlySet<string> => {
  const known = prs.filter((pr) => pr.status === 'open').map((pr) => pr.id).join(',');
  const load = useCallback(async () => {
    const ids = known === '' ? [] : known.split(',');
    const runs = await Promise.all(ids.map((pr) => readChecks(boardKey, { pr })));
    return failingPrIds(ids, runs);
  }, [boardKey, known]);
  const read = useRemoteRead(load, known === '' ? 0 : CHECK_POLL_MS);
  return read.data ?? NO_FAILING;
};

export { usePrFailing };
