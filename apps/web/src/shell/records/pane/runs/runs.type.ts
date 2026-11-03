/* @layer renderer-app @kind types */
import type { RunEntry } from '../../../../data/runs';
import type { LauncherView } from '../../../../screens/board/tabs/builds';
import type { RecordData } from '../pane.type';

interface RunsViewProps {
  data: RecordData;
  view: LauncherView;
  runs: RunEntry[];
}

interface BuildCommandProps {
  view: LauncherView;
  boardKey: string;
  runs: RunEntry[];
}

interface RunRowsProps {
  data: RecordData;
  runs: readonly RunEntry[];
}

export type { BuildCommandProps, RunRowsProps, RunsViewProps };
