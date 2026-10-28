/* @layer renderer-app @kind component */
import { CopyButton, Flex, FoldText, Text } from '@drizztdourden08/tessera';
import { threadOf } from '../model';
import type { CellProps } from '../worktrees.type';

const PathCell = (props: CellProps) => {
  const { row, context } = props;
  const thread = threadOf(row, context.meta);
  return (
    <Flex direction="column" gap="2xs">
      <Text as="div" variant="caption" tone="dim" mono weight={600}>
        <Flex gap="xs" align="center">
          <FoldText fill truncate="middle">{row.path}</FoldText>
          <CopyButton text={row.path} label="Copy" showLabel size="xs" />
        </Flex>
      </Text>
      <Text variant="caption" tone="muted">{`rule ${row.rule ?? '-'}${thread ? ` · tells ${thread}` : ''}`}</Text>
    </Flex>
  );
};

export { PathCell };
