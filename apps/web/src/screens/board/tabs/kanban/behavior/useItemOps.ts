/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { SOFT_INVERSE, createAction, createItem, criterionOp, editDoc, moveItem, softOp } from '../../../../../data/kanban';
import type { ItemRef, SoftKind } from '../../../../../data/kanban';
import { SOFT_LABEL } from '../kanban.constants';
import type { ItemOps } from '../kanban.type';
import { useWriter } from './useWriter';

const useItemOps = (boardKey: string): ItemOps => {
  const write = useWriter();
  return useMemo(() => {
    const soft = (kind: SoftKind, item: ItemRef) =>
      write(
        () => softOp(boardKey, { kind, id: item.id }),
        `${SOFT_LABEL[kind]} ${item.id}.`,
        (undoes) => void write(() => softOp(boardKey, { kind: SOFT_INVERSE[kind], id: item.id, undoes }), `Undone: ${item.id}.`),
      );
    return {
      move: (item, to, message) => write(() => moveItem(boardKey, item, to), message),
      setField: (item, field, value, message) =>
        write(() => editDoc(boardKey, { c: 'items', id: item.id, rev: item.rev }, { field, prev: item[field] ?? '', value }), message ?? `Saved ${item.id}.`),
      editRecord: (ref, change, message) => write(() => editDoc(boardKey, ref, change), message),
      criterion: (item, op, message) => write(() => criterionOp(boardKey, item, op), message),
      create: (item) => write(() => createItem(boardKey, item), `Adding "${item.title}".`),
      addTodo: (item, text) => write(() => createAction(boardKey, { text, item: item.id, thread: item.thread }), `To-do added for ${item.thread}.`),
      soft,
    };
  }, [boardKey, write]);
};

export { useItemOps };
