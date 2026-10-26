/* @layer renderer-app @kind logic */
import { useCallback } from 'react';
import { opVerdict } from '../../../../../data/kanban';
import type { SentOp } from '../../../../../data/kanban';
import type { Writer } from '../kanban.type';
import { announce } from './announce';
import { announceFailure } from './announce-failure';

const useWriter = (): Writer =>
  useCallback(async (send: () => Promise<SentOp>, done: string, undo?: (sentId: string) => void): Promise<boolean> => {
    try {
      const sent = await send();
      const verdict = opVerdict(sent.result, done);
      if (!verdict.ok || done) announce(verdict, sent.id, undo ? () => undo(sent.id) : undefined);
      return verdict.ok;
    } catch (error) {
      announceFailure(error);
      return false;
    }
  }, []);

export { useWriter };
