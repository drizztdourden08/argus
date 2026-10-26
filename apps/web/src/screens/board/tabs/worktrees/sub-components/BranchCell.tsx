/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { isLeftover } from '../model';
import type { CellProps } from '../worktrees.type';
import { LeftoverSize } from './LeftoverSize';

const BranchCell = (props: CellProps) => {
  const { row } = props;
  if (isLeftover(row)) return <LeftoverSize {...props} />;
  return (
    <Flex direction="column" gap="2xs">
      {row.branch && <Text.Code>{row.branch}</Text.Code>}
      {row.head && <Text variant="caption" tone="muted">{row.head}</Text>}
    </Flex>
  );
};

export { BranchCell };
