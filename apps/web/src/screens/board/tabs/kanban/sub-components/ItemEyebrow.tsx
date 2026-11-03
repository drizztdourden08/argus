/* @layer renderer-app @kind component */
import { Flex, Tag, Text } from '@drizztdourden08/tessera';
import type { ItemEyebrowProps } from '../drawer-props.type';
import { HEAD_ID_CLASS } from '../head.constants';
import { NEEDS_YOU_TIP } from '../kanban.constants';
import { ItemTypeMark } from './ItemTypeMark';
import { StatusTag } from './StatusTag';
import './HeadRow.css';

const ItemEyebrow = (props: ItemEyebrowProps) => {
  const { item, waiting } = props;
  return (
    <Flex gap="sm" align="center" wrap>
      <Flex gap="xs" align="center">
        <ItemTypeMark item={item} />
        <Text as="span" variant="caption" weight={600} mono className={HEAD_ID_CLASS}>{item.id}</Text>
      </Flex>
      <StatusTag status={item.status} />
      {waiting && <Tag variant="urgency" color="danger" title={NEEDS_YOU_TIP}>needs you</Tag>}
      {item.archivedAt && <Tag>archived</Tag>}
    </Flex>
  );
};

export { ItemEyebrow };
