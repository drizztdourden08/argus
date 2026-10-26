/* @layer renderer-app @kind component */
import { DateText, Flex, Tag, Text } from '@drizztdourden08/tessera';
import type { CardMetaProps } from '../OwnerFeed.type';

const CardMeta = (props: CardMetaProps) => {
  const { id, at, thread, item } = props;
  return (
    <Flex gap="xs" align="center" wrap>
      <Text variant="caption" tone="muted" mono>{id}</Text>
      {thread ? <Text variant="caption" tone="muted">{thread}</Text> : null}
      {item ? <Tag>{item}</Tag> : null}
      <Text variant="caption" tone="muted"><DateText value={at} format="relative" /></Text>
    </Flex>
  );
};

export { CardMeta };
