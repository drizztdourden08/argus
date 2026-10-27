/* @layer renderer-app @kind component */
import { DateText, Flex, Text } from '@drizztdourden08/tessera';
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
        <DateText value={at} format="elapsed" length="short" />
        {' ago'}
      </Text>
    </Flex>
  );
};

export { TouchedCell };
