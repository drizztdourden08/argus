/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { SOFT_INVERSE, editDoc, softOp } from '../../../../data/kanban';
import type { SoftKind } from '../../../../data/kanban';
import { SOFT_LABEL, useWriter } from '../../shared';
import { useOwnerPost } from '../../owner-posts';
import type { ThreadDoc, ThreadOps } from '../ThreadDrawer.type';

const useThreadOps = (boardKey: string): ThreadOps => {
  const write = useWriter();
  const poster = useOwnerPost(boardKey);
  return useMemo(() => {
    const soft = (kind: SoftKind, thread: ThreadDoc) =>
      void write(
        () => softOp(boardKey, { kind, id: thread.id, c: 'threads' }),
        `${SOFT_LABEL[kind]} ${thread.id}.`,
        (undoes) => void write(() => softOp(boardKey, { kind: SOFT_INVERSE[kind], id: thread.id, c: 'threads', undoes }), `Undone: ${thread.id}.`),
      );
    return {
      isBusy: poster.isBusy,
      setPaused: (thread, paused) => {
        const kind = paused ? 'pause' : 'resume';
        void poster.submit({
          key: `post:${kind}:${thread.id}`,
          fields: { kind, text: '', replyTo: thread.id, thread: thread.id },
          said: `${thread.id} ${paused ? 'paused' : 'resumed'}.`,
        });
      },
      setField: (thread, field, value) =>
        void write(() => editDoc(boardKey, { c: 'threads', id: thread.id, rev: thread.rev }, { field, prev: thread[field] ?? '', value }), `Saved ${thread.id}.`),
      soft,
    };
  }, [boardKey, write, poster]);
};

export { useThreadOps };
