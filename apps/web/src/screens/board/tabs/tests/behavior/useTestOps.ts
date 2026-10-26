/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { SOFT_INVERSE, sendOp } from '../../../../../data/kanban';
import type { DocData } from '../../../../../data/api';
import type { SoftKind } from '../../../../../data/kanban';
import { SOFT_LABEL } from '../../kanban/kanban.constants';
import { useWriter } from '../../kanban/behavior/useWriter';
import { previousValues } from '../model';
import type { EditChange, TestDoc } from '../model';
import type { TestOps } from '../tests.type';

const targetOf = (test: TestDoc): DocData => ({ c: 'tests', id: test.id });

const useTestOps = (boardKey: string): TestOps => {
  const write = useWriter();
  return useMemo(() => {
    const soft = (kind: SoftKind, test: TestDoc) =>
      write(
        () => sendOp(boardKey, { kind, target: targetOf(test) }),
        `${SOFT_LABEL[kind]} ${test.id}.`,
        (undoes) => void write(() => sendOp(boardKey, { kind: SOFT_INVERSE[kind], target: targetOf(test), undoes }), `Undone: ${test.id}.`),
      );
    const edit = (test: TestDoc, set: Partial<EditChange>) => {
      const fields = Object.keys(set) as (keyof EditChange)[];
      return write(() => sendOp(boardKey, { kind: 'edit', target: targetOf(test), set, prev: previousValues(test, fields), baseRev: test.rev }), `Saved ${test.id}.`);
    };
    const move = (test: TestDoc, item: string) =>
      write(() => sendOp(boardKey, { kind: 'move', target: targetOf(test), to: { item }, prev: { item: test.item }, baseRev: test.rev }), `Moved ${test.id}.`);
    return { edit, move, soft };
  }, [boardKey, write]);
};

export { useTestOps };
