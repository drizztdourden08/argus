/* @layer renderer-app @kind logic */
import type { Entry, OpOutcome, OwnerOp } from '../../../../../data/kanban';
import type { WithId } from '../../../../../data/react';
import { BAD_STATUSES, HUMAN, OP_KINDS } from './model.constants';
import type { TestNotice } from './model.type';

interface Slot {
  op: OwnerOp;
  result: OpOutcome | undefined;
  id: string;
  bulk: boolean;
}

const slotsOf = (entry: WithId<Entry>): Slot[] =>
  entry.kind === 'bulk'
    ? (entry.ops ?? []).map((op, index) => ({ op, result: entry.results?.[index], id: `${entry.id}#${index}`, bulk: true }))
    : [{ op: entry, result: undefined, id: entry.id, bulk: false }];

const waiting = (entry: Entry): boolean => !entry.appliedAt && (!entry.status || entry.status === 'new');

const statusOf = (entry: Entry, slot: Slot): string | undefined => slot.result?.status ?? (waiting(entry) ? 'new' : entry.status);

const outcomeOf = (entry: Entry, result: OpOutcome | undefined): OpOutcome => {
  const said = result ?? entry;
  return { outcome: said.outcome, current: said.current, by: said.by };
};

const noticeOf = (entry: WithId<Entry>, slot: Slot): TestNotice | null => {
  const status = statusOf(entry, slot) ?? '';
  if (!BAD_STATUSES.includes(status) || slot.op.target?.c !== 'tests') return null;
  return {
    id: slot.id,
    entryId: entry.id,
    bulk: slot.bulk,
    kind: status === 'conflict' ? 'conflict' : 'refused',
    op: slot.op,
    result: outcomeOf(entry, slot.result),
  };
};

const isOwnerOp = (entry: Entry): boolean => entry.author === HUMAN && OP_KINDS.includes(entry.kind) && entry.kind !== 'purge';

const ackedOps = (entries: readonly WithId<Entry>[]): Set<string> => {
  const acked = new Set<string>();
  for (const entry of entries) if (entry.author === HUMAN && entry.kind === 'ack' && entry.replyTo) acked.add(entry.replyTo);
  return acked;
};

const oldestFirst = (a: WithId<Entry>, b: WithId<Entry>): number => String(a.createdAt).localeCompare(String(b.createdAt)) || a.id.localeCompare(b.id);

const testNotices = (entries: readonly WithId<Entry>[], dismissed: ReadonlySet<string>): TestNotice[] => {
  const acked = ackedOps(entries);
  const open = entries.filter((entry) => isOwnerOp(entry) && !acked.has(entry.id)).sort(oldestFirst);
  const notices: TestNotice[] = [];
  for (const entry of open) {
    for (const slot of slotsOf(entry)) {
      const notice = noticeOf(entry, slot);
      if (notice && !dismissed.has(notice.id)) notices.push(notice);
    }
  }
  return notices;
};

export { testNotices };
