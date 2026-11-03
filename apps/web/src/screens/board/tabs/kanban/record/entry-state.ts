/* @layer renderer-app @kind logic */
import type { Entry as BoardEntry } from '../../../../../data/types';
import { ownerMarks } from '../../../feed-model';
import type { EntryState } from './record.type';

const entryState = (docs: readonly BoardEntry[]): EntryState => {
  const entries = docs.filter((entry) => !entry.deletedAt && !entry.archivedAt);
  const marks = ownerMarks(entries);
  const settled = new Set([...marks.answered, ...marks.acked]);
  const opened = new Set(entries.filter((entry) => entry.kind === 'question' && entry.status === 'open' && !settled.has(entry.id)).map((entry) => entry.id));
  return { entries, marks, opened };
};

export { entryState };
