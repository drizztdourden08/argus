/* @layer renderer-app @kind logic */
import type { PaneBadge, PaneData } from '../pane.type';

const discussionBadge = (data: PaneData): PaneBadge | undefined => {
  const { questions, todos } = data.record;
  const waiting = questions.filter((row) => row.open).length + todos.filter((todo) => !todo.done).length;
  return waiting > 0 ? { value: waiting, tone: 'warn', title: `${waiting} open questions and to-dos` } : undefined;
};

export { discussionBadge };
