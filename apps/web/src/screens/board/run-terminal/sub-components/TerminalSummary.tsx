/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import type { RunStatusProps } from '../../run-play';
import { terminalLast, terminalMeta } from '../model';

const TerminalSummary = (props: RunStatusProps) => {
  const { run } = props;
  return (
    <Flex gap="sm" align="center" wrap>
      <Text weight={600}>Terminal</Text>
      <Text variant="caption" tone="muted">{terminalMeta(run)}</Text>
      <Text variant="caption" tone="dim" mono>{terminalLast(run)}</Text>
    </Flex>
  );
};

export { TerminalSummary };
