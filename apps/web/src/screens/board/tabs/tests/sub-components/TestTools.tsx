/* @layer renderer-app @kind component */
import { DrawerToolbar, SoftTools } from '../../../shared';
import { useTestsShared } from '../behavior/useTestsShared';
import type { TestToolsProps } from '../tests-drawer-props.type';

const TestTools = (props: TestToolsProps) => {
  const { test, nav, wide } = props;
  const { ops } = useTestsShared();
  const archived = Boolean(test.archivedAt);
  return (
    <DrawerToolbar id={test.id} previous={nav.previous} next={nav.next} label="Test tools" wide={wide.wide} onWide={wide.toggle} onStep={nav.open}>
      <SoftTools archived={archived} onArchive={() => void ops.soft(archived ? 'unarchive' : 'archive', test)} onDelete={() => void ops.soft('delete', test)} />
    </DrawerToolbar>
  );
};

export { TestTools };
