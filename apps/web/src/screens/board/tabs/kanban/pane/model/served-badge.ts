/* @layer renderer-app @kind logic */
import type { PaneBadge, PaneData } from '../pane.type';

const servedBadge = (data: PaneData): PaneBadge | undefined => {
  const served = data.item.servedBy;
  const total = (served?.builds ?? 0) + (served?.worktrees ?? 0);
  if (!served || total === 0) return undefined;
  return { value: total, title: `${served.builds} live builds and ${served.worktrees} worktrees serve this item` };
};

export { servedBadge };
