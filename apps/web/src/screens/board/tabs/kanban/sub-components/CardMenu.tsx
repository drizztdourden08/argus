/* @layer renderer-app @kind component */
import { DropdownMenu } from '@drizztdourden08/tessera';
import { cardMenuGroups } from '../behavior/card-menu-groups';
import type { CardMenuProps } from '../kanban-props.type';

const CardMenu = (props: CardMenuProps) => {
  const label = `More actions for ${props.item.id}`;
  return (
    <DropdownMenu
      label={label}
      trigger={{ label, icon: 'ellipsis-vertical', iconOnly: true }}
      groups={cardMenuGroups(props)}
      variant="ghost"
      size="xs"
      align="end"
    />
  );
};

export { CardMenu };
