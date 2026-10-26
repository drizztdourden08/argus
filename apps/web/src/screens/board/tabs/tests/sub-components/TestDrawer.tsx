/* @layer renderer-app @kind component */
import { Drawer } from '@drizztdourden08/tessera';
import type { TestDrawerProps } from '../tests-drawer-props.type';
import { DrawerActions } from './DrawerActions';
import { TestBody } from './TestBody';

const TestDrawer = (props: TestDrawerProps) => {
  const { test, item, handed, nav, wide, notices, onClose } = props;
  const subtitle = test ? [test.id, test.covers].filter(Boolean).join(' / ') : undefined;
  return (
    <Drawer
      open={test !== null}
      onClose={onClose}
      side={wide.wide ? 'top' : 'right'}
      title={test?.title}
      subtitle={subtitle}
      label="Test details"
      actions={test ? <DrawerActions test={test} nav={nav} wide={wide} /> : undefined}
    >
      {test && <TestBody test={test} item={item} handed={handed} notices={notices} />}
    </Drawer>
  );
};

export { TestDrawer };
