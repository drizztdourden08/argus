/* @layer renderer-app @kind logic */
import type { Entry as BoardEntry } from '../../../../../data/types';
import { byCreated } from '../model';
import { REPLY_KINDS } from './record.constants';
import type { EntryRow } from './record.type';

const entryRow = (entry: BoardEntry, entries: readonly BoardEntry[], opened: ReadonlySet<string>): EntryRow => ({
  entry,
  open: opened.has(entry.id),
  replies: entries.filter((reply) => reply.replyTo === entry.id && REPLY_KINDS.includes(reply.kind)).sort(byCreated),
});

export { entryRow };
