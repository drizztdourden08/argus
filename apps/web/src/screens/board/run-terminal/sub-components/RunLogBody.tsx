/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import { RUN_LOG_FULL } from '../../../../data/runs';
import { FULL_LOG_HEIGHT } from '../run-terminal.constants';
import type { RunStopButtonProps } from '../run-terminal.type';
import { RunAttachments } from './RunAttachments';
import { TerminalBody } from './TerminalBody';

const RunLogBody = (props: RunStopButtonProps) => {
  const { run, boardKey } = props;
  return (
    <Flex direction="column" gap="md">
      <TerminalBody key={run.id} run={run} boardKey={boardKey} limit={RUN_LOG_FULL} height={FULL_LOG_HEIGHT} />
      <RunAttachments key={`attach-${run.id}`} boardKey={boardKey} runId={run.id} />
    </Flex>
  );
};

export { RunLogBody };
