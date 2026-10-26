/* @layer renderer-app @kind logic */
import { toast } from '@drizztdourden08/tessera';
import { useMemo } from 'react';
import type { ItemRef } from '../../../../../data/kanban';
import { replyFields, textProblem, useOwnerPost } from '../../../owner-posts';
import type { NoteKind } from '../../../owner-posts';
import { NOTE_WORD } from '../kanban.constants';
import type { RecordActions } from '../kanban.type';

const useRecordActions = (boardKey: string): RecordActions => {
  const { isBusy, submit } = useOwnerPost(boardKey);
  return useMemo(
    () => ({
      isBusy,
      done: (todo) =>
        submit({
          key: `post:done:${todo.id}`,
          fields: replyFields({ id: todo.id, item: todo.item, thread: todo.thread }, 'done', ''),
          said: `${todo.id} marked done.`,
        }),
      compose: (item: ItemRef, kind: NoteKind, text: string) => {
        const body = text.trim();
        const problem = body === '' ? `Write the ${NOTE_WORD[kind]}.` : textProblem(body);
        if (problem) {
          toast({ variant: 'danger', message: problem });
          return Promise.resolve(false);
        }
        const said = kind === 'decision' ? `Decision queued for ${item.thread}.` : `Note sent to ${item.thread}.`;
        return submit({ key: `compose:${item.id}`, fields: { kind, text: body, item: item.id, thread: item.thread }, said });
      },
    }),
    [isBusy, submit],
  );
};

export { useRecordActions };
