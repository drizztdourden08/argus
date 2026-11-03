/* @layer renderer-app @kind types */
import type { RunEntry } from '../../../../data/runs';
import type { GroupThreads } from '../../item-groups';
import type { BuildGroups, LauncherViews } from './model';

interface BuildsData {
  ready: boolean;
  views: LauncherViews;
  groups: BuildGroups;
  runs: RunEntry[];
  threads: GroupThreads;
}

interface RetiredFlag {
  shown: boolean;
  set: (open: boolean) => void;
}

export type { BuildsData, RetiredFlag };
