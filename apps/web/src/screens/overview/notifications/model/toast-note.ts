/* @layer renderer-app @kind logic */
import { noteLevel } from './note-level';
import type { NoteItem, ToastNote } from './model.type';

const withWhy = (prefix: string, why: string, join: string): string => (why ? `${prefix}${join}${why}` : prefix);

const toastNote = (note: NoteItem): ToastNote => {
  const why = typeof note.why === 'string' ? note.why : '';
  if (note.toast === 'shown') return { text: why ? `toast shown (${why})` : 'toast shown', bad: false };
  if (note.toast === 'failed') return { text: withWhy('toast failed', why, ': '), bad: true };
  if (note.toast === 'limited') return { text: why ? `not shown (limited: ${why})` : 'not shown (limited)', bad: false };
  if (noteLevel(note) === 'info') return { text: 'not shown (info is recorded only)', bad: false };
  const reason = /notify\.os/.test(why) ? 'disabled' : why || 'skipped';
  return { text: `not shown (${reason})`, bad: false };
};

export { toastNote };
