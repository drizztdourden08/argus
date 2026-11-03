/* @layer renderer-app @kind component */
import { isLiveRun, runLine } from '../../../../data/runs';
import { RecordDrawer } from '../../shared';
import type { RunLogDrawerProps } from '../run-terminal.type';
import { RunLogBody } from './RunLogBody';
import { RunStopButton } from './RunStopButton';

const RunLogDrawer = (props: RunLogDrawerProps) => {
  const { run, boardKey, onClose } = props;
  return (
    <RecordDrawer
      open={run !== null}
      label="Run log"
      wide={false}
      eyebrow={run && runLine(run).label}
      title={run ? `${run.name} ${run.id}` : ''}
      tools={run && isLiveRun(run) ? <RunStopButton run={run} boardKey={boardKey} /> : undefined}
      onClose={onClose}
    >
      {run && <RunLogBody run={run} boardKey={boardKey} />}
    </RecordDrawer>
  );
};

export { RunLogDrawer };
