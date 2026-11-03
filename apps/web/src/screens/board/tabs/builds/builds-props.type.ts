/* @layer renderer-app @kind types */
import type { RunEntry } from '../../../../data/runs';
import type { GroupThreads } from '../../item-groups';
import type { RetiredFlag } from './builds.type';
import type { BuildGroups, LauncherView } from './model';

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
}

export type {
  BuildFootProps, BuildSectionsProps, BuildSignalsProps, BuildsLeadProps, BuildsTabProps, GroupTitleProps, LauncherCardProps, LauncherListProps, LauncherPartProps, MasterGroupProps,
  RetiredNoteProps, RetiredSectionProps,
};
