/* @layer renderer-app @kind component */
import { Icon, IconButton } from '@drizztdourden08/tessera';
import type { ColumnAddProps } from '../kanban-props.type';

const ColumnAdd = (props: ColumnAddProps) => {
  const { title, open, onToggle } = props;
  return (
    <IconButton size="xs" variant="ghost" label={`Add a work item to ${title}`} aria-expanded={open} onClick={onToggle}>
      <Icon name="plus" />
    </IconButton>
  );
};

export { ColumnAdd };
