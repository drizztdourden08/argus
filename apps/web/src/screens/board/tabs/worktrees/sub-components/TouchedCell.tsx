/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { Age } from '../../../../../shell';
import { isLeftover } from '../model';
import type { CellProps } from '../worktrees.type';

const TouchedCell = (props: CellProps) => {
  const { row } = props;
  const leftover = isLeftover(row);
  const at = leftover ? row.leftover?.newestAt : row.lastTouchedAt;
  if (!at) return null;
  return (
    <Text variant="caption" tone="dim" mono numeric>
      {leftover ? 'last changed ' : null}
      <Age value={at} ago />
    </Text>
  );
};

export { TouchedCell };
