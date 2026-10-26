/* @layer renderer-app @kind component */
import { Flex, Status, Text } from '@drizztdourden08/tessera';
import { itemStatus } from '../behavior/item-status';
import { ITEM_STATUSES } from '../WorkItems.constants';
import type { ItemHeaderProps } from '../WorkItems.type';
import { NeedTags } from './NeedTags';

const ItemHeader = (props: ItemHeaderProps) => {
  const { item, need } = props;
  const criteria = item.ac ?? [];
  return (
    <Flex as="span" gap="sm" align="center" wrap>
      <Text as="span" variant="caption" tone="muted" mono>{item.id}</Text>
      <Status map={ITEM_STATUSES} value={itemStatus(item.status)} />
      <Text as="span">{item.title}</Text>
      <Text as="span" variant="caption" tone="muted">{item.thread}</Text>
      {criteria.length > 0 ? (
        <Text as="span" variant="caption" tone="muted">{criteria.filter((c) => c.done).length}/{criteria.length} criteria</Text>
      ) : null}
      <NeedTags need={need} />
    </Flex>
  );
};

export { ItemHeader };
