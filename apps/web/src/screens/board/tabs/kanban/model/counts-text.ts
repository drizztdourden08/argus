/* @layer renderer-app @kind logic */
import type { KanbanView } from './model.type';

const countsText = (view: KanbanView): string => {
  const parts = [`${view.open} open / ${view.total}`];
  if (view.filtered) parts.push('filtered');
  return parts.join(' · ');
};

export { countsText };
