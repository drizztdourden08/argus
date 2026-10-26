/* @layer renderer-app @kind types */
import type { RunEntry } from '../../../../data/runs';
import type { LauncherViews, RunGroups } from './model';

interface BuildsData {
  ready: boolean;
  views: LauncherViews;
  groups: RunGroups;
  runs: RunEntry[];
}

interface RetiredFlag {
  shown: boolean;
  set: (open: boolean) => void;
}

export type { BuildsData, RetiredFlag };
