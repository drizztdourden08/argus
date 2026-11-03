/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { useCollection } from '../../../../../../../data/react';
import type { Build, LaunchDef, Launcher, RunDoc } from '../../../../../../../data/types';
import type { Worktree } from '../../../../../../../data/worktrees';
import { launcherViews } from '../../../../builds';
import { useNow } from '../../../../../shared';
import { buildsOf } from '../model/builds-of';
import { worktreesOf } from '../model/worktrees-of';
import type { Served } from '../served.type';

const useServed = (boardKey: string, itemId: string): Served => {
  const launchers = useCollection<Launcher>(boardKey, 'launchers');
  const defs = useCollection<LaunchDef>(boardKey, 'launches');
  const runs = useCollection<RunDoc>(boardKey, 'runs');
  const bundles = useCollection<Build>(boardKey, 'builds');
  const worktrees = useCollection<Worktree>(boardKey, 'worktrees');
  const now = useNow(false);
  const views = useMemo(
    () => launcherViews({ launchers: launchers.docs, defs: defs.docs, runs: runs.docs, builds: bundles.docs, now }),
    [launchers.docs, defs.docs, runs.docs, bundles.docs, now],
  );
  return {
    ready: launchers.ready && worktrees.ready,
    builds: buildsOf(views, itemId),
    worktrees: worktreesOf(worktrees.docs, itemId),
    runs: runs.docs,
  };
};

export { useServed };
