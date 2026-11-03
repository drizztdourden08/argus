/* @layer renderer-app @kind component */
import type { ItemDrawerProps } from '../drawer-props.type';
import { ItemPane } from '../pane';
import { RecordDrawer } from '../../../shared';
import { DrawerNav } from './DrawerNav';
import { ItemEyebrow } from './ItemEyebrow';
import { ItemTitle } from './ItemTitle';

const ItemDrawer = (props: ItemDrawerProps) => {
  const { boardKey, item, items, tests, threads, actions, waiting, blocking, now, ops, nav, wide, onWide, onClose } = props;
  return (
    <RecordDrawer
      open={item !== null}
      label="Work item"
      wide={wide}
      eyebrow={item && <ItemEyebrow item={item} waiting={waiting} />}
      title={item && <ItemTitle item={item} ops={ops} />}
      tools={item && <DrawerNav item={item} ops={ops} nav={nav} wide={wide} onWide={onWide} />}
      onClose={onClose}
    >
      {item && (
        <ItemPane
          boardKey={boardKey}
          item={item}
          items={items}
          tests={tests}
          threads={threads}
          actions={actions}
          waiting={waiting}
          blocking={blocking}
          now={now}
          ops={ops}
          onOpen={nav.onStep}
        />
      )}
    </RecordDrawer>
  );
};

export { ItemDrawer };
