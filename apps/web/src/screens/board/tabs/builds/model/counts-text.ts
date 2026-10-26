/* @layer renderer-app @kind logic */
import type { LauncherViews } from './model.type';

const countsText = (views: LauncherViews): string => {
  if (views.live.length === 0) return '';
  const parts = [`${views.live.length} to start`];
  if (views.staleCount > 0) parts.push(`${views.staleCount} stale`);
  if (views.retired.length > 0) parts.push(`${views.retired.length} retired`);
  return parts.join(' · ');
};

export { countsText };
