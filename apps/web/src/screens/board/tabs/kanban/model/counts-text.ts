/* @layer renderer-app @kind logic */
import type { KanbanView } from './model.type';

const countsText = (view: KanbanView): string => {
  const parts = [`${view.open} open / ${view.total}`];
  if (view.filtered) parts.push('filtered');
  if (view.quiet > 0) parts.push(`${view.quiet} quiet`);
  return parts.join(' · ');
};

export { countsText };
