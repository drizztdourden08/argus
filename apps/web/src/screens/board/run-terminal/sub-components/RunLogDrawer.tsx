/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import { isLiveRun, runLine, RUN_LOG_FULL } from '../../../../data/runs';
import { RecordDrawer } from '../../shared';
import { FULL_LOG_HEIGHT } from '../run-terminal.constants';
import type { RunLogDrawerProps } from '../run-terminal.type';
import { RunAttachments } from './RunAttachments';
import { RunStopButton } from './RunStopButton';
import { TerminalBody } from './TerminalBody';

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
      {run && (
        <Flex direction="column" gap="md">
          <TerminalBody key={run.id} run={run} boardKey={boardKey} limit={RUN_LOG_FULL} height={FULL_LOG_HEIGHT} />
          <RunAttachments key={`attach-${run.id}`} boardKey={boardKey} runId={run.id} />
        </Flex>
      )}
    </RecordDrawer>
  );
};

export { RunLogDrawer };
