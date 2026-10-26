/* @layer renderer-app @kind logic */
import type { Entry as BoardEntry } from '../../../../../data/types';
import { ownerMarks } from '../../../feed-model';
import { byCreated } from '../model';
import { ENTRY_KINDS, REPLY_KINDS } from './record.constants';
import { byRank } from './by-rank';
import { timelineOf } from './timeline-of';
import type { EntryRow, ItemRecord, RecordInput } from './record.type';

const entryRow = (entry: BoardEntry, entries: readonly BoardEntry[], opened: ReadonlySet<string>): EntryRow => ({
  entry,
  open: opened.has(entry.id),
  replies: entries.filter((reply) => reply.replyTo === entry.id && REPLY_KINDS.includes(reply.kind)).sort(byCreated),
});

const isOpenQuestion = (entry: BoardEntry, settled: ReadonlySet<string>): boolean =>
  entry.kind === 'question' && entry.status === 'open' && !settled.has(entry.id);

const itemRecord = (input: RecordInput): ItemRecord => {
  const { itemId, actions, log } = input;
  const entries = input.entries.filter((entry) => !entry.deletedAt && !entry.archivedAt);
  const marks = ownerMarks(entries);
  const settled = new Set([...marks.answered, ...marks.acked]);
  const own = entries.filter((entry) => entry.item === itemId && ENTRY_KINDS.includes(entry.kind)).sort(byCreated);
  const opened = new Set(own.filter((entry) => isOpenQuestion(entry, settled)).map((entry) => entry.id));
  const todos = actions
    .filter((action) => action.item === itemId && !action.deletedAt && !action.archivedAt)
    .sort(byRank)
    .map((action) => ({ action, done: !(action.status === 'pending' && !marks.done.has(action.id)) }));
  return {
    questions: own.filter((entry) => entry.kind === 'question').map((entry) => entryRow(entry, entries, opened)),
    decisions: own.filter((entry) => entry.kind === 'decision').map((entry) => entryRow(entry, entries, opened)),
    todos,
    timeline: timelineOf({ itemId, entries: entries.filter((entry) => entry.item === itemId), opened, todos, log }),
  };
};

export { itemRecord };
