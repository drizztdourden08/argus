/* @layer renderer-app @kind component */
import { Link, Text } from '@drizztdourden08/tessera';
import { RUNNING_HINT } from '../BoardHeader.constants';
import type { BoardRunningProps } from '../BoardHeader.type';

const BoardRunning = (props: BoardRunningProps) => (
  <Text variant="caption" tone="muted">
    <Link href="#agents" variant="subtle" title={RUNNING_HINT}>{`${props.runningAgents} running`}</Link>
  </Text>
);

export { BoardRunning };
