/* @layer renderer-app @kind types */
import type { RunEntry } from '../../../../data/runs';
import type { OpenLog } from '../../run-terminal';
import type { RetiredFlag } from './builds.type';
import type { LauncherView, LauncherViews, RunGroups } from './model';

interface BuildsTabProps {
  boardKey: string;
}

interface RunScope {
  boardKey: string;
  runs: RunEntry[];
}

interface BuildsHeaderProps {
  views: LauncherViews;
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

interface RunsSectionProps extends OpenLog {
  groups: RunGroups;
  boardKey: string;
}

interface RunListProps extends OpenLog {
  title: string;
  runs: RunEntry[];
  boardKey: string;
}

interface RunRowProps extends OpenLog {
  run: RunEntry;
  boardKey: string;
}

export type {
  BuildsHeaderProps, BuildsTabProps, LauncherCardProps, LauncherListProps, LauncherPartProps, RetiredSectionProps, RunListProps, RunRowProps, RunsSectionProps,
};
