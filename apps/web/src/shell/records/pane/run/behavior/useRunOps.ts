/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { planRunOp } from '../../../../../data/kanban';
import { useWriter } from '../../../../../screens/board/shared';
import type { RunOps } from '../run.type';

const useRunOps = (boardKey: string, planId: string): RunOps => {
  const write = useWriter();
  return useMemo(
    () => ({
      start: () => write(() => planRunOp(boardKey, planId, { do: 'start' }), `Started a run of ${planId}.`),
      record: (test, status, note) => write(() => planRunOp(boardKey, planId, { do: 'record', test, status, note }), `${test} recorded as ${status}.`),
      end: () => write(() => planRunOp(boardKey, planId, { do: 'end' }), `Ended the run of ${planId}.`),
    }),
    [boardKey, planId, write],
  );
};

export { useRunOps };
