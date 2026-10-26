/* @layer renderer-app @kind component */
import { Drawer, Flex } from '@drizztdourden08/tessera';
import { isLiveRun, runLine, RUN_LOG_FULL } from '../../../../../data/runs';
import { FULL_LOG_HEIGHT } from '../builds.constants';
import type { RunLogDrawerProps } from '../builds-props.type';
import { RunAttachments } from './RunAttachments';
import { RunStopButton } from './RunStopButton';
import { TerminalBody } from './TerminalBody';

const RunLogDrawer = (props: RunLogDrawerProps) => {
  const { run, boardKey, onClose } = props;
  return (
    <Drawer
      open={run !== null}
      onClose={onClose}
      label="Run log"
      title={run ? `${run.name} ${run.id}` : ''}
      subtitle={run ? runLine(run).label : undefined}
      actions={run && isLiveRun(run) ? <RunStopButton run={run} boardKey={boardKey} /> : undefined}
    >
      {run && (
        <Flex direction="column" gap="md">
          <TerminalBody key={run.id} run={run} boardKey={boardKey} limit={RUN_LOG_FULL} height={FULL_LOG_HEIGHT} />
          <RunAttachments key={`attach-${run.id}`} boardKey={boardKey} runId={run.id} />
        </Flex>
      )}
    </Drawer>
  );
};

export { RunLogDrawer };
