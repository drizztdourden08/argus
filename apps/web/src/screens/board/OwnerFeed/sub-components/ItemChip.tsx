/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { itemCode, RecordLink, TypeIcon } from '../../../../shell';
import { DEFAULT_ITEM_KIND } from '../../feed-model';
import type { ItemChipProps } from '../OwnerFeed.type';

const ItemChip = (props: ItemChipProps) => {
  const { id, index } = props;
  const item = index.get(id);
  return (
    <Flex gap="xs" align="center" data-item-chip={id}>
      <TypeIcon kind={item?.type ?? DEFAULT_ITEM_KIND} />
      <RecordLink id={id}>
        <Text as="span" variant="caption" clamp={1}>{item ? `${itemCode(item)} ${item.title}` : id}</Text>
      </RecordLink>
    </Flex>
  );
};

export { ItemChip };
