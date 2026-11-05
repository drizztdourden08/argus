/* @layer renderer-app @kind logic */
import type { BuildFilters } from './builds-view.type';
import type { LauncherView } from './model.type';

const passes = (picked: readonly string[], value: string): boolean => picked.length === 0 || picked.includes(value);

const stageId = (view: LauncherView): string => (view.launcher.retiredAt ? 'closed' : (view.launcher.stage ?? 'ready'));

const textOf = (view: LauncherView): string => {
  const { launcher } = view;
  return `${launcher.name} ${launcher.title ?? ''} ${launcher.kind ?? ''} ${launcher.branch ?? ''}`.toLowerCase();
};

const matchBuild = (view: LauncherView, filters: BuildFilters): boolean => {
  const query = filters.query.trim().toLowerCase();
  if (query && !textOf(view).includes(query)) return false;
  if (!passes(filters.item, view.launcher.for ?? '')) return false;
  if (!passes(filters.kind, view.launcher.kind ?? '')) return false;
  return passes(filters.stage, stageId(view));
};

export { matchBuild };
