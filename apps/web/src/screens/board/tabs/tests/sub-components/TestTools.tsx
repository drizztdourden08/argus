/* @layer renderer-app @kind component */
import { Icon, IconButton } from '@drizztdourden08/tessera';
import { DrawerToolbar } from '../../../shared';
import { useTestsShared } from '../behavior/useTestsShared';
import type { TestToolsProps } from '../tests-drawer-props.type';

const TestTools = (props: TestToolsProps) => {
  const { test, nav, wide } = props;
  const { ops } = useTestsShared();
  const archived = Boolean(test.archivedAt);
  return (
    <DrawerToolbar id={test.id} previous={nav.previous} next={nav.next} label="Test tools" wide={wide.wide} onWide={wide.toggle} onStep={nav.open}>
      <IconButton label={archived ? 'Unarchive' : 'Archive'} size="xs" variant="ghost" onClick={() => void ops.soft(archived ? 'unarchive' : 'archive', test)}>
        <Icon name={archived ? 'rotate-ccw' : 'archive'} />
      </IconButton>
      <IconButton label="Delete" title="Delete (Undo stays a few seconds)" size="xs" variant="ghost" tone="danger" onClick={() => void ops.soft('delete', test)}>
        <Icon name="trash-2" />
      </IconButton>
    </DrawerToolbar>
  );
};

export { TestTools };
