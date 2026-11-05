/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { Age } from '../../../../shell';
import { UPDATED_HINT } from '../BoardHeader.constants';
import type { BoardStampProps } from '../BoardHeader.type';

const BoardStamp = (props: BoardStampProps) => {
  const { status, updatedAt } = props;
  if (status === 'unavailable') return <Text variant="caption" tone="muted">dashboard unreachable</Text>;
  if (!updatedAt) return <Text variant="caption" tone="muted">waiting for the board</Text>;
  return <Text variant="caption" tone="muted" title={UPDATED_HINT}>updated <Age value={updatedAt} /></Text>;
};

export { BoardStamp };
