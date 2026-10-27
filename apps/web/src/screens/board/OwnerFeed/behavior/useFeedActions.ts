/* @layer renderer-app @kind logic */
import { toast } from '@drizztdourden08/tessera';
import { useMemo } from 'react';
import { softOp, SOFT_INVERSE } from '../../../../data/kanban';
import { contextRequestFields, replyFields, replyProblem, useOwnerPost } from '../../owner-posts';
import { SOFT_LABEL, useWriter } from '../../shared';
import type { FeedActions } from '../OwnerFeed.type';

const useFeedActions = (boardKey: string, handed: string | null = null): FeedActions => {
  const { isBusy, submit } = useOwnerPost(boardKey);
  const write = useWriter();
  return useMemo(
    () => ({
      isBusy,
      handed,
      soft: (kind, c, id) =>
        write(
          () => softOp(boardKey, { kind, c, id }),
          `${SOFT_LABEL[kind]} ${id}.`,
          (undoes) => void write(() => softOp(boardKey, { kind: SOFT_INVERSE[kind], c, id, undoes }), `Undone: ${id}.`),
        ),
      choose: (entry, option) =>
        submit({ key: `answer:${entry.id}`, fields: replyFields(entry, 'decision', option), said: `Decided: ${option}` }),
      reply: (entry, kind, text) => {
        const body = text.trim();
        const problem = replyProblem(body, kind);
        if (problem) {
          toast({ variant: 'danger', message: problem });
          return Promise.resolve(false);
        }
        const said = kind === 'decision' ? `Decision sent to ${entry.thread}.` : `Reply sent to ${entry.thread}.`;
        return submit({ key: `answer:${entry.id}`, fields: replyFields(entry, kind, body), said });
      },
      dismiss: (entry) =>
        submit({ key: `post:ack:${entry.id}`, fields: replyFields(entry, 'ack', ''), said: 'Dismissed.' }),
      done: (action) =>
        submit({ key: `post:done:${action.id}`, fields: replyFields(action, 'done', ''), said: `${action.id} marked done.` }),
      askContext: (entry) =>
        submit({ key: `askctx:${entry.id}`, fields: contextRequestFields(entry), said: `Asked ${entry.thread} for context.` }),
    }),
    [boardKey, handed, isBusy, submit, write],
  );
};

export { useFeedActions };
