/* @layer renderer-app @kind types */
import type { RunEntry } from '../../../../data/runs';
import type { RetiredFlag } from './builds.type';
import type { LauncherView, LauncherViews, RunGroups } from './model';

interface BuildsTabProps {
  boardKey: string;
}

interface OpenLog {
  onLog: (runId: string) => void;
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

interface LogButtonProps extends OpenLog {
  run: RunEntry | null;
}

interface RunTerminalProps extends OpenLog {
  run: RunEntry;
  boardKey: string;
}

interface TerminalBodyProps {
  run: RunEntry;
  boardKey: string;
  limit: number;
  height: number;
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

interface RunStopButtonProps {
  run: RunEntry;
  boardKey: string;
}

interface RunLogDrawerProps {
  run: RunEntry | null;
  boardKey: string;
  onClose: () => void;
}

interface RunAttachmentsProps {
  boardKey: string;
  runId: string;
}

export type {
  BuildsHeaderProps, BuildsTabProps, LauncherCardProps, LauncherListProps, LauncherPartProps, LogButtonProps, RetiredSectionProps,
  RunAttachmentsProps, RunListProps, RunLogDrawerProps, RunRowProps, RunsSectionProps, RunStopButtonProps, RunTerminalProps, TerminalBodyProps,
};
