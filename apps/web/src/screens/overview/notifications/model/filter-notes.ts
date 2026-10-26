/* @layer renderer-app @kind logic */
import { noteLevel } from './note-level';
import { noteProject } from './note-project';
import type { NoteFilters, NoteItem } from './model.type';

const filterNotes = (notes: readonly NoteItem[], filters: NoteFilters): NoteItem[] =>
  notes.filter((note) => (filters.level === 'all' || noteLevel(note) === filters.level) && (!filters.project || noteProject(note) === filters.project));

export { filterNotes };
