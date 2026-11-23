/* @layer renderer-app @kind logic */
import { isLiveRun } from '../../../../../data/runs';
import type { TabStat } from '../../../shared';
import { NO_MASTER_TIP } from '../builds.constants';
import type { BuildGroups, LauncherViews } from './model.type';

const buildStats = (views: LauncherViews, groups: BuildGroups): TabStat[] => {
  const running = views.live.filter((view) => view.runs.some(isLiveRun)).length;
  const missing = views.hasMaster ? 0 : 1;
  return [
    { id: 'builds', label: 'builds', count: views.live.length, tone: 'neutral' },
    { id: 'running', label: 'running', count: running, tone: running > 0 ? 'info' : 'neutral' },
    { id: 'stale', label: 'stale', count: views.staleCount, tone: views.staleCount > 0 ? 'warning' : 'neutral', tip: 'Not used within its window' },
    { id: 'needs-link', label: 'needs link', count: groups.needsLink.length, tone: groups.needsLink.length > 0 ? 'warning' : 'neutral', tip: 'Serves no open item yet' },
    { id: 'no-master', label: 'no master', count: missing, tone: missing > 0 ? 'danger' : 'neutral', tip: NO_MASTER_TIP },
  ];
};

export { buildStats };
