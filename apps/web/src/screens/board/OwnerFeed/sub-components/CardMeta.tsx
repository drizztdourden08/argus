/* @layer renderer-app @kind component */
import { Flex, Tag, Text } from '@drizztdourden08/tessera';
import { Age } from '../../../../shell';
import type { CardMetaProps } from '../OwnerFeed.type';

const CardMeta = (props: CardMetaProps) => {
  const { label, id, at, thread, item, menu } = props;
  return (
    <Flex gap="sm" align="center" wrap>
      <Text variant="overline" tone="dim">{label}</Text>
      <Text variant="caption" tone="muted" mono>{id}</Text>
      {thread ? <Text variant="caption" tone="muted">{thread}</Text> : null}
      {item ? <Tag>{item}</Tag> : null}
      <Text variant="caption" tone="muted"><Age value={at} /></Text>
      {menu ? <Flex className="feed-card__menu">{menu}</Flex> : null}
    </Flex>
  );
};

export { CardMeta };
