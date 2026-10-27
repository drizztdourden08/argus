/* @layer renderer-app @kind component */
import { Icon, IconButton } from '@drizztdourden08/tessera';
import { DrawerToolbar } from '../../../shared';
import type { DrawerNavProps } from '../drawer-props.type';

const DrawerNav = (props: DrawerNavProps) => {
  const { item, ops, nav, wide, onWide } = props;
  const { previous, next, onStep } = nav;
  const archived = Boolean(item.archivedAt);
  return (
    <DrawerToolbar id={item.id} previous={previous} next={next} label="Item tools" wide={wide} onWide={onWide} onStep={onStep}>
      <IconButton
        label={archived ? 'Unarchive' : 'Archive'}
        size="xs"
        variant="ghost"
        onClick={() => void ops.soft(archived ? 'unarchive' : 'archive', item)}
      >
        <Icon name={archived ? 'rotate-ccw' : 'archive'} />
      </IconButton>
      <IconButton label="Delete" title="Delete (Undo stays a few seconds)" size="xs" variant="ghost" tone="danger" onClick={() => void ops.soft('delete', item)}>
        <Icon name="trash-2" />
      </IconButton>
    </DrawerToolbar>
  );
};

export { DrawerNav };
