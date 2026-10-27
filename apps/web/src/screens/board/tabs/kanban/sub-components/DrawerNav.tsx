/* @layer renderer-app @kind component */
import { DrawerToolbar, SoftTools } from '../../../shared';
import type { DrawerNavProps } from '../drawer-props.type';

const DrawerNav = (props: DrawerNavProps) => {
  const { item, ops, nav, wide, onWide } = props;
  const { previous, next, onStep } = nav;
  const archived = Boolean(item.archivedAt);
  return (
    <DrawerToolbar id={item.id} previous={previous} next={next} label="Item tools" wide={wide} onWide={onWide} onStep={onStep}>
      <SoftTools archived={archived} onArchive={() => void ops.soft(archived ? 'unarchive' : 'archive', item)} onDelete={() => void ops.soft('delete', item)} />
    </DrawerToolbar>
  );
};

export { DrawerNav };
