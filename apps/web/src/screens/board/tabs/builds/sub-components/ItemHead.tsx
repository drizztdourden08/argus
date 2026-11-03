/* @layer renderer-app @kind component */
import { Flex, Tag, Text } from '@drizztdourden08/tessera';
import { ItemTypeMark } from '../../kanban/sub-components/ItemTypeMark';
import type { ItemHeadProps } from '../builds-props.type';
import { stageText } from '../model';

const ItemHead = (props: ItemHeadProps) => {
  const { item } = props;
  return (
    <Flex gap="sm" align="center" wrap>
      <ItemTypeMark item={item} />
      <Tag pill icon="link" href={`#${item.id}`} title={`Open ${item.id} on the Kanban tab`}>{item.id}</Tag>
      <Text weight={600}>{item.title}</Text>
      <Tag title="Where the item stands">{stageText(item)}</Tag>
    </Flex>
  );
};

export { ItemHead };
