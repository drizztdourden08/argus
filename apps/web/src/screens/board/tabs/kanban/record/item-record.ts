/* @layer renderer-app @kind logic */
import { byCreated } from '../model';
import { ENTRY_KINDS } from './record.constants';
import { byRank } from './by-rank';
import { entryRow } from './entry-row';
import { entryState } from './entry-state';
import { timelineOf } from './timeline-of';
import type { ItemRecord, RecordInput } from './record.type';

const itemRecord = (input: RecordInput): ItemRecord => {
  const { itemId, actions, log } = input;
  const { entries, marks, opened } = entryState(input.entries);
  const own = entries.filter((entry) => entry.item === itemId && ENTRY_KINDS.includes(entry.kind)).sort(byCreated);
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
