/* @layer renderer-app @kind component */
import { Drawer, ScreenWindow, ScrollArea } from '@drizztdourden08/tessera';
import type { ItemDrawerProps } from '../drawer-props.type';
import { DrawerNav } from './DrawerNav';
import { ItemBody } from './ItemBody';
import { ItemTitle } from './ItemTitle';

const ItemDrawer = (props: ItemDrawerProps) => {
  const { boardKey, item, tests, threads, actions, now, ops, nav, wide, onWide, onClose } = props;
  const title = item && <ItemTitle item={item} ops={ops} />;
  const body = item && (
    <ItemBody
      boardKey={boardKey}
      item={item}
      tests={tests}
      threads={threads}
      actions={actions}
      now={now}
      ops={ops}
      tools={<DrawerNav item={item} ops={ops} nav={nav} wide={wide} onWide={onWide} />}
    />
  );
  if (item && wide) {
    return (
      <ScreenWindow title={title} subtitle={item.id} onClose={onClose}>
        <ScrollArea>{body}</ScrollArea>
      </ScreenWindow>
    );
  }
  return (
    <Drawer open={item !== null} onClose={onClose} label="Work item" title={title ?? undefined} subtitle={item?.id}>
      {body}
    </Drawer>
  );
};

export { ItemDrawer };
