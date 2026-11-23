/* @layer renderer-app @kind types */
import type { RunEntry } from '../../../../data/runs';
import type { GroupThreads } from '../../item-groups';
import type { BuildLayout, BuildsData } from './builds.type';
import type { TabView } from '../../BoardTabsRow';
import type { BuildFilters, BuildGroup, BuildGroups, BuildOrder, LauncherView } from './model';

interface BuildsTabProps {
  boardKey: string;
}

interface RunScope {
  boardKey: string;
  runs: RunEntry[];
  compact: boolean;
}

interface LauncherListProps extends RunScope {
  views: LauncherView[];
  label: string;
}

interface LauncherCardProps extends RunScope {
  view: LauncherView;
  threads: GroupThreads;
}

interface LauncherPartProps {
  view: LauncherView;
}

interface BuildFootProps extends RunScope {
  view: LauncherView;
}

interface BuildSignalsProps {
  view: LauncherView;
  code: string;
}

interface RetiredNoteProps extends LauncherPartProps {
  boardKey: string;
}

interface RetiredSectionProps extends RunScope {
  views: LauncherView[];
}

interface MasterGroupProps extends RunScope {
  views: LauncherView[];
}

interface GroupTitleProps {
  title: string;
  note: string;
}

interface BuildSectionsProps extends RunScope {
  groups: BuildGroups;
  threads: GroupThreads;
  group: BuildGroup;
  views: LauncherView[];
}

interface BuildFlatSectionsProps extends RunScope {
  groups: BuildGroups;
  group: BuildGroup;
  views: LauncherView[];
}

interface BuildsToolsProps {
  boardKey: string;
  layout: { value: BuildLayout; onChange: (layout: BuildLayout) => void };
  closed: { count: number; shown: boolean; onToggle: () => void };
  tabView: TabView<BuildGroup, BuildOrder, BuildFilters>;
  data: BuildsData;
}

interface BuildRunsProps {
  runs: readonly RunEntry[];
  boardKey: string;
  now: number;
}

interface BuildRunRowProps {
  run: RunEntry;
  boardKey: string;
  now: number;
}

export type {
  BuildFlatSectionsProps, BuildFootProps, BuildRunRowProps, BuildRunsProps, BuildSectionsProps, BuildSignalsProps, BuildsTabProps, BuildsToolsProps, GroupTitleProps, LauncherCardProps, LauncherListProps, LauncherPartProps, MasterGroupProps,
  RetiredNoteProps, RetiredSectionProps,
};
