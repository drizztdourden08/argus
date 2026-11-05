/* @layer renderer-app @kind types */
import type { RunEntry } from '../../../../data/runs';
import type { GroupThreads } from '../../item-groups';
import type { BuildsData, RetiredFlag } from './builds.type';
import type { TabView } from '../../BoardTabsRow/BoardTabsRow.type';
import type { BuildFilters, BuildGroup, BuildGroups, BuildOrder, LauncherView } from './model';

interface BuildsTabProps {
  boardKey: string;
}

interface BuildsLeadProps {
  counts: string;
}

interface RunScope {
  boardKey: string;
  runs: RunEntry[];
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
  flag: RetiredFlag;
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
  tabView: TabView<BuildGroup, BuildOrder, BuildFilters>;
  data: BuildsData;
}

export type {
  BuildFlatSectionsProps, BuildFootProps, BuildSectionsProps, BuildSignalsProps, BuildsLeadProps, BuildsTabProps, BuildsToolsProps, GroupTitleProps, LauncherCardProps, LauncherListProps, LauncherPartProps, MasterGroupProps,
  RetiredNoteProps, RetiredSectionProps,
};
