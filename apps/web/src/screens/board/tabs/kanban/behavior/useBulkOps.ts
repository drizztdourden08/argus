/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import type { DocData } from '../../../../../data/api';
import { bulkOp, opVerdict } from '../../../../../data/kanban';
import type { OpVerdict } from '../../../../../data/kanban';
import { bulkMoves, bulkSofts } from '../model';
import type { BulkOps } from '../kanban.type';
import { announce } from './announce';
import { announceFailure } from './announce-failure';

const changesText = (count: number): string => `${count} change${count === 1 ? '' : 's'} sent.`;

const useBulkOps = (boardKey: string): BulkOps =>
  useMemo(() => {
    const send = (ops: DocData[], inverse: DocData[] | null): void => {
      bulkOp(boardKey, ops).then((sent) => {
        const failed: OpVerdict | undefined = sent.results.map((result) => opVerdict(result, '')).find((verdict) => !verdict.ok);
        const undo = inverse ? () => void bulkOp(boardKey, inverse, sent.ids).then(() => announce({ ok: true, message: 'Undone.' }, `undo-${sent.ids.join('-')}`), announceFailure) : undefined;
        announce(failed ?? { ok: true, message: changesText(ops.length) }, sent.ids.join('-'), undo);
      }, announceFailure);
    };
    return {
      move: (items, to) => send(bulkMoves(items, to), null),
      soft: (kind, items) => {
        const { ops, inverse } = bulkSofts(kind, items);
        send(ops, inverse);
      },
    };
  }, [boardKey]);

export { useBulkOps };
