/* @layer renderer-app @kind component */
import { Drawer, Text } from '@drizztdourden08/tessera';
import type { ItemDrawerProps } from '../kanban-props.type';
import { DrawerActions } from './DrawerActions';
import { InlineEdit } from './InlineEdit';
import { ItemBody } from './ItemBody';

const ItemDrawer = (props: ItemDrawerProps) => {
  const { item, tests, threads, now, ops, onClose } = props;
  const title = item && (
    <InlineEdit
      label="title"
      field="title"
      value={item.title}
      display={<Text variant="title">{item.title}</Text>}
      onSave={(value) => void ops.setField(item, 'title', value)}
    />
  );
  return (
    <Drawer
      open={item !== null}
      onClose={onClose}
      label="Work item"
      title={title ?? undefined}
      subtitle={item?.id}
      actions={item ? <DrawerActions item={item} ops={ops} /> : undefined}
    >
      {item && <ItemBody item={item} tests={tests} threads={threads} now={now} ops={ops} />}
    </Drawer>
  );
};

export { ItemDrawer };
