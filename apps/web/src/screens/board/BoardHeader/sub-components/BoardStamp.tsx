/* @layer renderer-app @kind component */
import { DateText, Text } from '@drizztdourden08/tessera';
import type { BoardStampProps } from '../BoardHeader.type';

const BoardStamp = (props: BoardStampProps) => {
  const { status, updatedAt } = props;
  if (status === 'unavailable') return <Text variant="caption" tone="muted">dashboard unreachable</Text>;
  if (!updatedAt && (status === 'connecting' || status === 'idle')) return <Text variant="caption" tone="muted">connecting</Text>;
  if (!updatedAt) return <Text variant="caption" tone="muted">waiting for the board</Text>;
  return <Text variant="caption" tone="muted">board updated <DateText value={updatedAt} format="elapsed" length="short" /> ago</Text>;
};

export { BoardStamp };
