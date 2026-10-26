/* @layer renderer-app @kind logic */
import { useCallback, useMemo, useState } from 'react';
import { ackOp, resendOp } from '../../../../../data/kanban';
import type { Entry, ItemRef } from '../../../../../data/kanban';
import type { WithId } from '../../../../../data/react';
import { useWriter } from '../../../shared';
import type { OpNotice } from '../../../shared';
import { opNotices } from '../model';
import { purgeNotices } from '../trash';
import type { PurgeNotice } from '../trash';
import type { NoticeOps } from '../kanban.type';

const useNoticeOps = (boardKey: string, entries: readonly WithId<Entry>[], items: readonly ItemRef[]): NoticeOps => {
  const write = useWriter();
  const [dismissed, setDismissed] = useState<ReadonlySet<string>>(new Set());
  const notices = useMemo(() => opNotices(entries, dismissed), [entries, dismissed]);
  const purges = useMemo(() => purgeNotices(entries), [entries]);
  const dismissPurge = useCallback((notice: PurgeNotice) => void write(() => ackOp(boardKey, notice.id), 'Dismissed.'), [boardKey, write]);

  const dismiss = useCallback((notice: OpNotice) => {
    if (!notice.bulk) {
      void write(() => ackOp(boardKey, notice.id), 'Dismissed.');
      return;
    }
    const left = notices.filter((other) => other.entryId === notice.entryId && other.id !== notice.id);
    setDismissed((now) => new Set([...now, notice.id]));
    if (left.length === 0) void write(() => ackOp(boardKey, notice.entryId), 'Dismissed.');
  }, [boardKey, notices, write]);

  const resend = useCallback((notice: OpNotice) => {
    const item = items.find((candidate) => candidate.id === notice.op.target?.id);
    void write(() => resendOp(boardKey, { ...notice.op, ...notice.result }, notice.id, item?.rev), 'Sent again with force.');
    dismiss(notice);
  }, [boardKey, items, write, dismiss]);

  return { notices, purges, dismiss, dismissPurge, resend };
};

export { useNoticeOps };
