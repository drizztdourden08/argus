/* @layer renderer-app @kind types */
import type { ReactNode } from 'react';
import type { ItemRef } from '../../../../data/kanban';
import type { RunEntry } from '../../../../data/runs';
import type { OpenLog } from '../../run-terminal';
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

interface LauncherListProps extends OpenLog, RunScope {
  views: LauncherView[];
  label: string;
}

interface LauncherCardProps extends OpenLog, RunScope {
  view: LauncherView;
}

interface LauncherPartProps {
  view: LauncherView;
}

interface RetiredSectionProps extends OpenLog, RunScope {
  views: LauncherView[];
  flag: RetiredFlag;
}

interface BuildGroupProps extends OpenLog, RunScope {
  id: string;
  head: ReactNode;
  views: LauncherView[];
  label: string;
}

interface GroupTitleProps {
  title: string;
  note: string;
}

interface ItemHeadProps {
  item: ItemRef;
}

interface BuildSectionsProps extends OpenLog, RunScope {
  groups: BuildGroups;
}

export type {
  BuildGroupProps, BuildsLeadProps, BuildSectionsProps, BuildsTabProps, GroupTitleProps, ItemHeadProps, LauncherCardProps, LauncherListProps, LauncherPartProps, RetiredSectionProps,
};
