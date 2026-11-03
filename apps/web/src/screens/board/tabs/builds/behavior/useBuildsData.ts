/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { useCollection } from '../../../../../data/react';
import type { Item, ThreadChoice } from '../../../../../data/kanban';
import type { Build, LaunchDef, Launcher, RunDoc } from '../../../../../data/types';
import type { BuildsData } from '../builds.type';
import { groupByItem, launcherViews } from '../model';
import { useClock } from './useClock';

const useBuildsData = (boardKey: string): BuildsData => {
  const launchers = useCollection<Launcher>(boardKey, 'launchers');
  const defs = useCollection<LaunchDef>(boardKey, 'launches');
  const runs = useCollection<RunDoc>(boardKey, 'runs');
  const builds = useCollection<Build>(boardKey, 'builds');
  const items = useCollection<Item>(boardKey, 'items');
  const threads = useCollection<ThreadChoice>(boardKey, 'threads');
  const now = useClock();
  const views = useMemo(
    () => launcherViews({ launchers: launchers.docs, defs: defs.docs, runs: runs.docs, builds: builds.docs, now }),
    [launchers.docs, defs.docs, runs.docs, builds.docs, now],
  );
  const groups = useMemo(() => groupByItem(views, items.docs), [views, items.docs]);
  return { ready: launchers.ready && items.ready, views, groups, runs: runs.docs, threads: threads.docs };
};

export { useBuildsData };
