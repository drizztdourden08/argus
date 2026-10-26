/* @layer renderer-app @kind logic */
import { useCallback } from 'react';
import { purgeOp } from '../../../../../data/kanban';
import { PURGE_SENT } from '../kanban.constants';
import { purgeCriteria } from '../trash';
import type { PurgeCounts, PurgeDraft } from '../trash';
import { useWriter } from '../../../shared';

const usePurge = (boardKey: string): ((draft: PurgeDraft, counts: PurgeCounts) => Promise<boolean>) => {
  const write = useWriter();
  return useCallback(
    (draft, counts) => write(() => purgeOp(boardKey, { criteria: purgeCriteria(draft), expect: counts, confirm: draft.confirm }), PURGE_SENT),
    [boardKey, write],
  );
};

export { usePurge };
