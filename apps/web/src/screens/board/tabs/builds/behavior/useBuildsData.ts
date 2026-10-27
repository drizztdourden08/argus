/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { useCollection } from '../../../../../data/react';
import type { Build, LaunchDef, Launcher, RunDoc } from '../../../../../data/types';
import type { BuildsData } from '../builds.type';
import { launcherViews } from '../model';
import { useClock } from './useClock';

const useBuildsData = (boardKey: string): BuildsData => {
  const launchers = useCollection<Launcher>(boardKey, 'launchers');
  const defs = useCollection<LaunchDef>(boardKey, 'launches');
  const runs = useCollection<RunDoc>(boardKey, 'runs');
  const builds = useCollection<Build>(boardKey, 'builds');
  const now = useClock();
  const views = useMemo(
    () => launcherViews({ launchers: launchers.docs, defs: defs.docs, runs: runs.docs, builds: builds.docs, now }),
    [launchers.docs, defs.docs, runs.docs, builds.docs, now],
  );
  return { ready: launchers.ready, views, runs: runs.docs };
};

export { useBuildsData };
