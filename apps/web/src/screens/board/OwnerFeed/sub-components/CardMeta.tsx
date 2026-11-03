/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { Age } from '../../../../shell';
import { EntryLink } from '../../shared';
import type { CardMetaProps } from '../OwnerFeed.type';

const CardMeta = (props: CardMetaProps) => {
  const { label, id, linked, at, thread, item, menu } = props;
  return (
    <Flex gap="sm" align="center" justify="between">
      <Flex gap="sm" align="center" wrap>
        <Text variant="overline" tone="dim">{label}</Text>
        {linked ? <EntryLink id={id} /> : <Text variant="caption" tone="muted" mono>{id}</Text>}
        {thread ? <Text variant="caption" tone="muted">{thread}</Text> : null}
        {item ? <Text variant="caption" tone="dim" mono weight={600}>{item}</Text> : null}
        <Text variant="caption" tone="muted"><Age value={at} /></Text>
      </Flex>
      {menu}
    </Flex>
  );
};

export { CardMeta };
