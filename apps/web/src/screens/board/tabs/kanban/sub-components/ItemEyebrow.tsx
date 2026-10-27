/* @layer renderer-app @kind component */
import { Tag, Text } from '@drizztdourden08/tessera';
import type { ItemEyebrowProps } from '../drawer-props.type';
import { NEEDS_YOU_TIP } from '../kanban.constants';
import { StatusTag } from './StatusTag';

const ItemEyebrow = (props: ItemEyebrowProps) => {
  const { item, waiting } = props;
  return (
    <>
      <Text variant="caption" weight={600} mono>{item.id}</Text>
      <StatusTag status={item.status} />
      {waiting && <Tag variant="urgency" color="danger" title={NEEDS_YOU_TIP}>needs you</Tag>}
      {item.archivedAt && <Tag>archived</Tag>}
    </>
  );
};

export { ItemEyebrow };
