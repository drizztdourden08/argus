/* @layer renderer-app @kind logic */
import { noteLevel } from './note-level';
import type { LevelPick, NoteItem } from './model.type';

const countLevels = (notes: readonly NoteItem[]): Record<LevelPick, number> => {
  const counts: Record<LevelPick, number> = { all: notes.length, info: 0, ping: 0, urgent: 0 };
  for (const note of notes) counts[noteLevel(note)] += 1;
  return counts;
};

export { countLevels };
