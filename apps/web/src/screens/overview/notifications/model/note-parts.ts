/* @layer renderer-app @kind logic */
import type { NoteItem, NoteParts } from './model.type';

const noteParts = (note: NoteItem): NoteParts => {
  const title = note.title ?? '';
  const text = note.text ?? '';
  return title === '' ? { title: text, detail: '' } : { title, detail: text };
};

export { noteParts };
