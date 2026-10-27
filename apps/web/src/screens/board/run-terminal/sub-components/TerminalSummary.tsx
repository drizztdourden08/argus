/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import type { RunStatusProps } from '../../run-play';
import { terminalLast, terminalMeta } from '../model';

const TerminalSummary = (props: RunStatusProps) => {
  const { run } = props;
  return (
    <>
      <Text variant="caption" tone="muted">{terminalMeta(run)}</Text>
      <Text variant="caption" tone="dim" mono>{terminalLast(run)}</Text>
    </>
  );
};

export { TerminalSummary };
