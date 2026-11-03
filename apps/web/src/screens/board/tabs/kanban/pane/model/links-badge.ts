/* @layer renderer-app @kind logic */
import type { PaneBadge, PaneData } from '../pane.type';

const linksBadge = (data: PaneData): PaneBadge | undefined => {
  const waiting = data.item.blocked?.length ?? 0;
  return waiting > 0 ? { value: waiting, tone: 'warn', title: `${waiting} links hold this item back` } : undefined;
};

export { linksBadge };
