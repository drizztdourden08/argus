/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { SOFT_INVERSE, createItem, criterionOp, editItem, moveItem, softOp } from '../../../../../data/kanban';
import type { ItemRef, SoftKind } from '../../../../../data/kanban';
import { SOFT_LABEL } from '../kanban.constants';
import type { ItemOps } from '../kanban.type';
import { useWriter } from './useWriter';

const useItemOps = (boardKey: string): ItemOps => {
  const write = useWriter();
  return useMemo(() => {
    const soft = (kind: SoftKind, item: ItemRef) =>
      write(
        () => softOp(boardKey, kind, item.id),
        `${SOFT_LABEL[kind]} ${item.id}.`,
        (undoes) => void write(() => softOp(boardKey, SOFT_INVERSE[kind], item.id, undoes), `Undone: ${item.id}.`),
      );
    return {
      move: (item, to, message) => write(() => moveItem(boardKey, item, to), message),
      setField: (item, field, value, message) => write(() => editItem(boardKey, item, field, value), message ?? `Saved ${item.id}.`),
      criterion: (item, op, message) => write(() => criterionOp(boardKey, item, op), message),
      create: (item) => write(() => createItem(boardKey, item), `Adding "${item.title}".`),
      soft,
    };
  }, [boardKey, write]);
};

export { useItemOps };
