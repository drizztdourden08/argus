/* @layer renderer-app @kind logic */
import type { RunStatus } from '../../../../../../../data/kanban';
import { NEEDS_NOTE, NOTE_MAX } from '../run.constants';

const noteProblem = (status: RunStatus, note: string): string | null => {
  if (note.length > NOTE_MAX) return `The note holds ${NOTE_MAX} characters at most.`;
  if (NEEDS_NOTE.includes(status) && note.trim() === '') return `A note is required for ${status}.`;
  return null;
};

export { noteProblem };
