/* @layer renderer-app @kind logic */
import type { Entry as BoardEntry } from '../../../../../data/types';
import { entryRow } from './entry-row';
import { entryState } from './entry-state';
import type { EntryRow } from './record.type';

const entryOf = (id: string | null, docs: readonly BoardEntry[]): EntryRow | null => {
  const { entries, opened } = entryState(docs);
  const entry = entries.find((candidate) => candidate.id === id);
  return entry ? entryRow(entry, entries, opened) : null;
};

export { entryOf };
