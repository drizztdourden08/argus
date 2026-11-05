/* @layer renderer-app @kind types */
import type { ItemRef } from '../../../../data/kanban';
import type { RunEntry } from '../../../../data/runs';
import type { GroupThreads } from '../../item-groups';
import type { BuildGroups, LauncherViews } from './model';

interface BuildsData {
  ready: boolean;
  views: LauncherViews;
  groups: BuildGroups;
  items: ItemRef[];
  runs: RunEntry[];
  threads: GroupThreads;
}

interface RetiredFlag {
  shown: boolean;
  set: (open: boolean) => void;
}

export type { BuildsData, RetiredFlag };
