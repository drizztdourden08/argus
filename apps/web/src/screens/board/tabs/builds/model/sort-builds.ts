/* @layer renderer-app @kind logic */
import type { SortDirection } from '@drizztdourden08/tessera';
import { MASTER } from '../builds.constants';
import type { BuildOrder } from './builds-view.type';
import type { LauncherView } from './model.type';

const pinRank = (view: LauncherView): number => {
  if (view.launcher.name === MASTER) return 0;
  return view.launcher.current ? 1 : 2;
};

const byName = (a: LauncherView, b: LauncherView): number => a.launcher.name.localeCompare(b.launcher.name, undefined, { numeric: true });

const byUpdated = (a: LauncherView, b: LauncherView): number => String(a.launcher.updatedAt ?? '').localeCompare(String(b.launcher.updatedAt ?? ''));

const sortBuilds = (views: readonly LauncherView[], order: BuildOrder, direction: SortDirection): LauncherView[] => {
  const compare = order === 'name' ? byName : byUpdated;
  const sign = direction === 'desc' ? -1 : 1;
  return [...views].sort((a, b) => pinRank(a) - pinRank(b) || sign * compare(a, b) || byName(a, b));
};

export { sortBuilds };
