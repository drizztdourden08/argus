/* @layer renderer-app @kind component */
import { Drawer } from '@drizztdourden08/tessera';
import type { TestDrawerProps } from '../tests-props.type';
import { TestBody } from './TestBody';

const TestDrawer = (props: TestDrawerProps) => {
  const { test, item, boardKey, handed, onClose } = props;
  const subtitle = test ? [test.id, test.covers].filter(Boolean).join(' / ') : undefined;
  return (
    <Drawer open={test !== null} onClose={onClose} title={test?.title} subtitle={subtitle} label="Test details">
      {test && <TestBody test={test} item={item} boardKey={boardKey} handed={handed} />}
    </Drawer>
  );
};

export { TestDrawer };
