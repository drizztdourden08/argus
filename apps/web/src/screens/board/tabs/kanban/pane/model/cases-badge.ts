/* @layer renderer-app @kind logic */
import type { PaneBadge, PaneData } from '../pane.type';
import { caseIds } from './case-ids';

const casesBadge = (data: PaneData): PaneBadge | undefined => {
  const count = caseIds(data.item).length;
  return count > 0 ? { value: count, title: `${count} cases` } : undefined;
};

export { casesBadge };
