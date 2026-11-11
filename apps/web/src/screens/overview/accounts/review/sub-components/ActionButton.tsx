/* @layer renderer-app @kind component */
import { Icon, IconButton } from '@drizztdourden08/tessera';
import type { ActionButtonProps } from '../review-tree-props.type';

const ActionButton = (props: ActionButtonProps) => {
  const { label, icon, danger, onClick } = props;
  return (
    <IconButton variant="ghost" size="xs" tone={danger ? 'danger' : undefined} label={label} tooltip={label} onClick={onClick}>
      <Icon name={icon} />
    </IconButton>
  );
};

export { ActionButton };
