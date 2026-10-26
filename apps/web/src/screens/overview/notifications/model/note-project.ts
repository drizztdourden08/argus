/* @layer renderer-app @kind logic */
import type { NoteItem } from './model.type';

const noteProject = (note: NoteItem): string => (typeof note.project === 'string' && note.project ? note.project : '');

export { noteProject };
