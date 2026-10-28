/* @layer renderer-app @kind component */
import { DropdownMenu } from '@drizztdourden08/tessera';
import { copyText } from '../../shared';
import type { CardMenuProps } from '../OwnerFeed.type';

const CardMenu = (props: CardMenuProps) => {
  const { collection, id, actions } = props;
  return (
    <DropdownMenu
      label={`More actions for ${id}`}
      trigger={{ label: `More actions for ${id}`, icon: 'ellipsis-vertical', iconOnly: true }}
      groups={[
        {
          id: 'record',
          items: [
            { id: 'archive', label: 'Archive', onSelect: () => void actions.soft('archive', collection, id) },
            { id: 'delete', label: 'Delete', tone: 'danger', onSelect: () => void actions.soft('delete', collection, id) },
            { id: 'copy', label: 'Copy id', onSelect: () => copyText(id) },
          ],
        },
      ]}
      variant="ghost"
      size="xs"
      align="end"
      itemSize="roomy"
    />
  );
};

export { CardMenu };
