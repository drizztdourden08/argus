/* @layer renderer-app @kind logic */
import type { BuildViewSpec } from './builds-view.type';
import { matchBuild } from './match-build';
import type { LauncherView, LauncherViews } from './model.type';
import { sortBuilds } from './sort-builds';

const buildsView = (views: LauncherViews, spec: BuildViewSpec): LauncherViews => {
  const shape = (list: readonly LauncherView[]) => sortBuilds(list.filter((view) => matchBuild(view, spec.filters)), spec.order, spec.direction);
  return { ...views, live: shape(views.live), retired: shape(views.retired) };
};

export { buildsView };
