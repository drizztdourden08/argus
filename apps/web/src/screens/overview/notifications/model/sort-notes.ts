/* @layer renderer-app @kind logic */
import { asNumber, toMillis } from '../../../../format';
import type { NoteItem } from './model.type';

const newestFirst = (a: NoteItem, b: NoteItem): number =>
  (asNumber(b.n) ?? 0) - (asNumber(a.n) ?? 0) || (toMillis(b.at) || 0) - (toMillis(a.at) || 0);

const sortNotes = (notes: readonly NoteItem[]): NoteItem[] => [...notes].sort(newestFirst);

export { sortNotes };
