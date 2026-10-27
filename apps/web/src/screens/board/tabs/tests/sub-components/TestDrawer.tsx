/* @layer renderer-app @kind component */
import { RecordDrawer } from '../../../shared';
import type { TestDrawerProps } from '../tests-drawer-props.type';
import { TestBody } from './TestBody';
import { TestTools } from './TestTools';

const TestDrawer = (props: TestDrawerProps) => {
  const { test, item, handed, nav, wide, notices, onClose } = props;
  return (
    <RecordDrawer
      open={test !== null}
      label="Test details"
      wide={wide.wide}
      eyebrow={test && [test.id, test.covers].filter(Boolean).join(' · ')}
      title={test?.title}
      tools={test && <TestTools test={test} nav={nav} wide={wide} />}
      onClose={onClose}
    >
      {test && <TestBody test={test} item={item} handed={handed} notices={notices} />}
    </RecordDrawer>
  );
};

export { TestDrawer };
