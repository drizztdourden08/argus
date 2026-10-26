/* @layer renderer-app @kind component */
import { Drawer } from '@drizztdourden08/tessera';
import type { ItemDrawerProps } from '../drawer-props.type';
import { DrawerNav } from './DrawerNav';
import { ItemBody } from './ItemBody';
import { ItemTitle } from './ItemTitle';

const ItemDrawer = (props: ItemDrawerProps) => {
  const { boardKey, item, tests, threads, actions, waiting, now, ops, nav, wide, onWide, onClose } = props;
  const title = item && <ItemTitle item={item} ops={ops} />;
  const body = item && (
    <ItemBody
      boardKey={boardKey}
      item={item}
      tests={tests}
      threads={threads}
      actions={actions}
      waiting={waiting}
      now={now}
      ops={ops}
      tools={<DrawerNav item={item} ops={ops} nav={nav} wide={wide} onWide={onWide} />}
    />
  );
  return (
    <Drawer open={item !== null} onClose={onClose} size={wide ? 'xl' : 'md'} label="Work item" title={title ?? undefined} subtitle={item?.id}>
      {body}
    </Drawer>
  );
};

export { ItemDrawer };
