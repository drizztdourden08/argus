/* @layer renderer-app @kind logic */
import { useCallback, useMemo, useState } from 'react';
import { ackOp, resendOp } from '../../../../../data/kanban';
import type { Entry } from '../../../../../data/kanban';
import type { WithId } from '../../../../../data/react';
import { useWriter } from '../../../shared';
import { testNotices } from '../model';
import type { TestDoc, TestNotice } from '../model';
import type { NoticeOps } from '../tests.type';

const useTestNotices = (boardKey: string, entries: readonly WithId<Entry>[], tests: readonly TestDoc[]): NoticeOps => {
  const write = useWriter();
  const [dismissed, setDismissed] = useState<ReadonlySet<string>>(new Set());
  const notices = useMemo(() => testNotices(entries, dismissed), [entries, dismissed]);

  const dismiss = useCallback((notice: TestNotice) => {
    const left = notices.filter((other) => other.entryId === notice.entryId && other.id !== notice.id);
    if (notice.bulk) setDismissed((now) => new Set([...now, notice.id]));
    if (!notice.bulk || left.length === 0) void write(() => ackOp(boardKey, notice.entryId), 'Dismissed.');
  }, [boardKey, notices, write]);

  const resend = useCallback((notice: TestNotice) => {
    const test = tests.find((candidate) => candidate.id === notice.op.target?.id);
    void write(() => resendOp(boardKey, { ...notice.op, ...notice.result }, notice.id, test?.rev), 'Sent again with force.');
    dismiss(notice);
  }, [boardKey, tests, write, dismiss]);

  return { notices, dismiss, resend };
};

export { useTestNotices };
