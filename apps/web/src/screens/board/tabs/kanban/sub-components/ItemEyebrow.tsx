/* @layer renderer-app @kind component */
import { Flex, Tag } from '@drizztdourden08/tessera';
import type { ItemEyebrowProps } from '../drawer-props.type';
import { NEEDS_YOU_TIP } from '../kanban.constants';
import { StatusTag } from './StatusTag';

const ItemEyebrow = (props: ItemEyebrowProps) => {
  const { item, waiting } = props;
  return (
    <Flex gap="sm" align="center" wrap>
      <StatusTag status={item.status} />
      {waiting && <Tag variant="urgency" color="danger" title={NEEDS_YOU_TIP}>needs you</Tag>}
      {item.archivedAt && <Tag>archived</Tag>}
    </Flex>
  );
};

export { ItemEyebrow };
