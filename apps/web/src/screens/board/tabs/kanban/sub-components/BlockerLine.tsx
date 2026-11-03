/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { RecordLink } from '../../../../../shell';
import type { BlockerLineProps } from '../drawer-props.type';

const BlockerLine = (props: BlockerLineProps) => {
  const { blocker, link } = props;
  if (!link) return <Text variant="caption" tone="muted">{`blocked: ${blocker}`}</Text>;
  return (
    <Flex gap="xs" align="center">
      <Text variant="caption" tone="muted">blocked on</Text>
      <RecordLink id={link.id} />
    </Flex>
  );
};

export { BlockerLine };
