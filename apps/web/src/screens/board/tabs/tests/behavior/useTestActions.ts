/* @layer renderer-app @kind logic */
import { toast } from '@drizztdourden08/tessera';
import { useCallback } from 'react';
import { handBuild, rerunTest } from '../../../../../data/posts';
import type { TestActions } from '../tests.type';
import { useOwnerPost } from './useOwnerPost';

const useTestActions = (boardKey: string): TestActions => {
  const { busy, run } = useOwnerPost();
  const report = useCallback((failure: string | null) => {
    if (failure) toast({ variant: 'danger', message: failure });
  }, []);
  const rerun = useCallback((testId: string) => run(() => rerunTest(boardKey, testId), `${testId} is back in To run.`).then(report), [boardKey, run, report]);
  const hand = useCallback((tag: string | null) => {
    const done = tag ? `Build ${tag} is now handed to you.` : 'No build is handed to you now.';
    return run(() => handBuild(boardKey, tag), done).then(report);
  }, [boardKey, run, report]);
  return { busy, rerun, hand };
};

export { useTestActions };
