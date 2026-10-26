/* @layer renderer-app @kind component */
import { Button } from '@drizztdourden08/tessera';
import type { DrawerActionsProps } from '../kanban-props.type';

const DrawerActions = (props: DrawerActionsProps) => {
  const { item, ops } = props;
  return (
    <>
      <Button variant="secondary" onClick={() => void ops.soft(item.archivedAt ? 'unarchive' : 'archive', item)}>
        {item.archivedAt ? 'Unarchive' : 'Archive'}
      </Button>
      <Button variant="danger" onClick={() => void ops.soft('delete', item)}>Delete</Button>
    </>
  );
};

export { DrawerActions };
