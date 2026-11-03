/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { Age, RecordLink } from '../../../../shell';
import type { CardMetaProps } from '../OwnerFeed.type';

const CardMeta = (props: CardMetaProps) => {
  const { label, id, at, thread, item, menu } = props;
  return (
    <Flex gap="sm" align="center" justify="between">
      <Flex gap="sm" align="center" wrap>
        <Text variant="overline" tone="dim">{label}</Text>
        <RecordLink id={id} />
        {thread ? <Text variant="caption" tone="muted">{thread}</Text> : null}
        {item ? <RecordLink id={item}><Text as="span" variant="caption" tone="dim" mono weight={600}>{item}</Text></RecordLink> : null}
        <Text variant="caption" tone="muted"><Age value={at} /></Text>
      </Flex>
      {menu}
    </Flex>
  );
};

export { CardMeta };
