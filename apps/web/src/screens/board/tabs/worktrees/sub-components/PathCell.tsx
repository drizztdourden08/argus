/* @layer renderer-app @kind component */
import { CopyButton, Flex, Text } from '@drizztdourden08/tessera';
import { threadOf } from '../model';
import type { CellProps } from '../worktrees.type';
import { CellText } from './CellText';

const PathCell = (props: CellProps) => {
  const { row, context } = props;
  const thread = threadOf(row, context.meta);
  return (
    <Flex direction="column" gap="2xs">
      <Flex gap="xs" align="center">
        <CellText tail={row.path.length - row.path.lastIndexOf('/')} variant="caption" tone="dim" mono weight={600} title={row.path}>{row.path}</CellText>
        <CopyButton text={row.path} label="Copy" showLabel size="xs" />
      </Flex>
      <Text variant="caption" tone="muted">{`rule ${row.rule ?? '-'}${thread ? ` · tells ${thread}` : ''}`}</Text>
    </Flex>
  );
};

export { PathCell };
