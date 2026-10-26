/* @layer renderer-app @kind logic */
import type { Entry, OpOutcome, OwnerOp } from '../../../../../data/kanban';
import type { WithId } from '../../../../../data/react';
import { byCreated } from './by-created';
import { isUnapplied } from './is-unapplied';
import { BAD_STATUSES, HUMAN, OP_KINDS } from './model.constants';
import type { OpNotice } from './model.type';

const ackedIds = (entries: readonly WithId<Entry>[]): Set<string> =>
  new Set(entries.filter((entry) => entry.author === HUMAN && entry.kind === 'ack' && entry.replyTo).map((entry) => String(entry.replyTo)));

const statusOf = (entry: Entry, result: OpOutcome | undefined): string | undefined => {
  if (result?.status) return result.status;
  return isUnapplied(entry) ? 'new' : entry.status;
};

const outcomeOf = (entry: Entry, result: OpOutcome | undefined): OpOutcome => ({
  outcome: result?.outcome ?? entry.outcome,
  current: result?.current ?? entry.current,
  by: result?.by ?? entry.by,
});

const noticeOf = (entry: WithId<Entry>, op: OwnerOp, result: OpOutcome | undefined, index: number | null): OpNotice | null => {
  const status = statusOf(entry, result);
  if (!status || !BAD_STATUSES.includes(status) || op.target?.c !== 'items') return null;
  const bulk = index !== null;
  return {
    id: bulk ? `${entry.id}#${index}` : entry.id,
    entryId: entry.id,
    bulk,
    kind: status === 'conflict' ? 'conflict' : 'refused',
    op,
    result: outcomeOf(entry, result),
  };
};

const opsOf = (entry: WithId<Entry>): OpNotice[] => {
  if (entry.kind !== 'bulk') return [noticeOf(entry, entry, undefined, null)].filter((notice) => notice !== null);
  const notices = (entry.ops ?? []).map((op, index) => noticeOf(entry, op, entry.results?.[index], index));
  return notices.filter((notice) => notice !== null);
};

const opNotices = (entries: readonly WithId<Entry>[], dismissed: ReadonlySet<string>): OpNotice[] => {
  const acked = ackedIds(entries);
  return entries
    .filter((entry) => entry.author === HUMAN && OP_KINDS.includes(entry.kind) && entry.kind !== 'purge' && !acked.has(entry.id))
    .sort(byCreated)
    .flatMap(opsOf)
    .filter((notice) => !dismissed.has(notice.id));
};

export { opNotices };
