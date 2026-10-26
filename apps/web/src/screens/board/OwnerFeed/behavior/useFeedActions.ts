/* @layer renderer-app @kind logic */
import { toast } from '@drizztdourden08/tessera';
import { useMemo } from 'react';
import { contextRequestFields, replyFields, replyProblem, useOwnerPost } from '../../owner-posts';
import type { FeedActions } from '../OwnerFeed.type';

const useFeedActions = (boardKey: string): FeedActions => {
  const { isBusy, submit } = useOwnerPost(boardKey);
  return useMemo(
    () => ({
      isBusy,
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
    [isBusy, submit],
  );
};

export { useFeedActions };
