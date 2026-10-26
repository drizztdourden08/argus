/* @layer renderer-app @kind logic */
import { KIND_LABEL } from '../record';
import { TRASH_COLLECTIONS } from './trash.constants';
import type { TrashInput, TrashKind, TrashRow, TrashSource } from './trash.type';

const first = (...values: (string | null | undefined)[]): string => values.find((value) => value !== undefined && value !== null && value !== '') ?? '';

const isTrashed = (c: TrashKind, doc: TrashSource): boolean => {
  if (c === 'agents') return first(doc.deletedAt, doc.archivedAt) !== '';
  if (c === 'entries') return first(doc.deletedAt) !== '' && first(doc.kind) in KIND_LABEL;
  return first(doc.deletedAt) !== '';
};

const textOf = (c: TrashKind, doc: TrashSource): string =>
  first(doc.title, doc.text, c === 'agents' ? `${first(doc.name, doc.id)}: ${first(doc.task)}` : '');

const rowOf = (c: TrashKind, doc: TrashSource): TrashRow => ({
  key: `${c}:${doc.id}`,
  c,
  id: doc.id,
  kind: c,
  text: textOf(c, doc),
  thread: first(doc.thread),
  at: first(doc.deletedAt, doc.archivedAt),
  by: first(doc.deletedBy, doc.updatedBy, 'unknown'),
  reason: first(doc.deleteReason),
  archived: first(doc.deletedAt) === '',
});

const trashRows = (input: TrashInput): TrashRow[] =>
  TRASH_COLLECTIONS.flatMap((c) => input[c].filter((doc) => isTrashed(c, doc)).map((doc) => rowOf(c, doc))).sort((a, b) => b.at.localeCompare(a.at));

export { trashRows };
