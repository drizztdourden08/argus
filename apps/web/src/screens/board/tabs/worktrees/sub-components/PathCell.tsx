/* @layer renderer-app @kind component */
import { CopyValue, Flex, Text } from '@drizztdourden08/tessera';
import { threadOf } from '../model';
import type { CellProps } from '../worktrees.type';

const PathCell = (props: CellProps) => {
  const { row, context } = props;
  const thread = threadOf(row, context.meta);
  return (
    <Flex direction="column" gap="2xs">
      <CopyValue value={row.path} label="path" mono truncate="middle" />
      <Text variant="caption" tone="muted">{`rule ${row.rule ?? '-'}${thread ? `, tells ${thread}` : ''}`}</Text>
    </Flex>
  );
};

export { PathCell };
