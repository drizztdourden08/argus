/* @layer renderer-app @kind component */
import { Button } from '@drizztdourden08/tessera';
import type { ColumnAddProps } from '../kanban-props.type';

const ColumnAdd = (props: ColumnAddProps) => {
  const { title, open, onToggle } = props;
  return (
    <Button size="xs" variant="ghost" aria-label={`Add a work item to ${title}`} aria-expanded={open} onClick={onToggle}>
      + Add
    </Button>
  );
};

export { ColumnAdd };
