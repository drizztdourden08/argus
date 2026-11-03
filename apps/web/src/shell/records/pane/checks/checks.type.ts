/* @layer renderer-app @kind types */
import type { CheckRun } from '../../../../data/remote';
import type { RecordData } from '../pane.type';

interface CheckRowsProps {
  data: RecordData;
  runs: readonly CheckRun[];
}

interface StepLogProps {
  boardKey: string;
  run: CheckRun;
  index: number;
  first: boolean;
}

interface CheckRunViewProps {
  data: RecordData;
  run: CheckRun | null;
  ready: boolean;
}

export type { CheckRowsProps, CheckRunViewProps, StepLogProps };
