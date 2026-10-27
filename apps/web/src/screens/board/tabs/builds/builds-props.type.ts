/* @layer renderer-app @kind types */
import type { RunEntry } from '../../../../data/runs';
import type { OpenLog } from '../../run-terminal';
import type { RetiredFlag } from './builds.type';
import type { LauncherView } from './model';

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

export type {
  BuildsLeadProps, BuildsTabProps, LauncherCardProps, LauncherListProps, LauncherPartProps, RetiredSectionProps,
};
