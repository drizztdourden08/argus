/* @layer renderer-app @kind component */
import { Flex, Tag, Text } from '@drizztdourden08/tessera';
import { NONE_WAITING } from '../OverlordChat.constants';
import type { QueueSectionProps } from '../OverlordChat.type';

const QueueSection = (props: QueueSectionProps) => {
  const { title, count, children } = props;
  return (
    <Flex direction="column" gap="sm">
      <Flex gap="sm" align="center">
        <Text variant="label">{title}</Text>
        <Tag>{count}</Tag>
      </Flex>
      {count === 0 ? <Text variant="caption" tone="muted">{NONE_WAITING}</Text> : children}
    </Flex>
  );
};

export { QueueSection };
