/* @layer renderer-app @kind logic */
import { noteProject } from './note-project';
import type { NoteItem } from './model.type';

const projectCounts = (notes: readonly NoteItem[]): [string, number][] => {
  const counts = new Map<string, number>();
  for (const note of notes) {
    const project = noteProject(note);
    if (project) counts.set(project, (counts.get(project) ?? 0) + 1);
  }
  return [...counts.entries()].sort(([a], [b]) => a.localeCompare(b));
};

export { projectCounts };
