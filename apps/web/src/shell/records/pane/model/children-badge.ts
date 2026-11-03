/* @layer renderer-app @kind logic */
import { CLOSED_STATUSES } from '../../../../screens/board/tabs/kanban/model';
import type { PaneBadge, PaneData } from '../pane.type';
import { childrenOf } from './children-of';

const childrenBadge = (data: PaneData): PaneBadge | undefined => {
  const open = childrenOf(data.items, data.item.id).filter((child) => !CLOSED_STATUSES.includes(child.status)).length;
  return open > 0 ? { value: open, title: `${open} open children` } : undefined;
};

export { childrenBadge };
