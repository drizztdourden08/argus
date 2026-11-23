/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { Age } from '../../../../../shell';
import type { WorktreeScannedProps } from '../worktrees-props.type';

const WorktreeScanned = (props: WorktreeScannedProps) => {
  const { scannedAt } = props;
  return (
    <Text variant="caption" tone="muted">
      {scannedAt ? <>scanned <Age value={scannedAt} /> ago</> : 'not scanned yet'}
    </Text>
  );
};

export { WorktreeScanned };
