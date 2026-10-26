/* @layer renderer-app @kind component */
import { Button } from '@drizztdourden08/tessera';
import type { ColumnAddProps } from '../kanban-props.type';

const ColumnAdd = (props: ColumnAddProps) => {
  const { title, onAdd } = props;
  return (
    <Button size="xs" variant="ghost" aria-label={`Add a work item to ${title}`} onClick={onAdd}>
      + Add
    </Button>
  );
};

export { ColumnAdd };
