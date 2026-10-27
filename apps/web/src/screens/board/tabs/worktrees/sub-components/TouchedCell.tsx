/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { Age } from '../../../../../shell';
import { isLeftover } from '../model';
import type { CellProps } from '../worktrees.type';

const TouchedCell = (props: CellProps) => {
  const { row } = props;
  const leftover = isLeftover(row);
  const at = leftover ? row.leftover?.newestAt : row.lastTouchedAt;
  if (!at) return null;
  return (
    <Flex direction="column" gap="2xs">
      {leftover && <Text variant="caption" tone="muted">last changed</Text>}
      <Text variant="caption" tone="dim" mono numeric>
        <Age value={at} />
        {' ago'}
      </Text>
    </Flex>
  );
};

export { TouchedCell };
