/* @layer renderer-app @kind logic */
import { toast } from '@drizztdourden08/tessera';
import { useCallback } from 'react';
import { editDoc } from '../../../../../data/kanban';
import { rerunTest } from '../../../../../data/posts';
import { OWNER_ASSIGNEE } from '../model';
import type { ItemDoc } from '../model';
import type { TestActions } from '../tests.type';
import { useOwnerPost } from './useOwnerPost';

const useTestActions = (boardKey: string): TestActions => {
  const { busy, run } = useOwnerPost();
  const report = useCallback((failure: string | null) => {
    if (failure) toast({ variant: 'danger', message: failure });
  }, []);
  const rerun = useCallback((testId: string) => run(() => rerunTest(boardKey, testId), `${testId} is back in To run.`).then(report), [boardKey, run, report]);
  const hand = useCallback((target: ItemDoc, handed: boolean) => {
    const done = handed ? `${target.id} is no longer handed to you.` : `${target.id} is now handed to you.`;
    const change = { field: 'assignee', prev: target.assignee ?? '', value: handed ? null : OWNER_ASSIGNEE };
    return run(() => editDoc(boardKey, { c: 'items', id: target.id }, change).then((sent) => sent.result), done).then(report);
  }, [boardKey, run, report]);
  return { busy, rerun, hand };
};

export { useTestActions };
