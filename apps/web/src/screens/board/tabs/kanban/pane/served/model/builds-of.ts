/* @layer renderer-app @kind logic */
import type { LauncherView, LauncherViews } from '../../../../builds';

const buildsOf = (views: LauncherViews, itemId: string): LauncherView[] =>
  [...views.live, ...views.retired].filter((view) => view.launcher.for === itemId);

export { buildsOf };
