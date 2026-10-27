/* @layer renderer-app @kind logic */
import type { MenuGroup, MenuItem } from '@drizztdourden08/tessera';
import type { CardMenuProps } from '../kanban-props.type';
import { copyText, threadChoices } from '../../../shared';
import type { Option } from '../../../shared';
import { STATUS_OPTIONS } from '../model';

const cardMenuGroups = (props: CardMenuProps): MenuGroup[] => {
  const { item, threads, ops, onOpen } = props;
  const { id } = item;
  const archive: MenuItem = item.archivedAt
    ? { id: 'unarchive', label: 'Unarchive', onSelect: () => void ops.soft('unarchive', item) }
    : { id: 'archive', label: 'Archive', onSelect: () => void ops.soft('archive', item) };
  const moveRow = (field: 'status' | 'thread', label: string, options: readonly Option[]): MenuItem => ({
    id: field,
    label,
    kind: 'select',
    options,
    selected: item[field],
    onPick: (value) => value !== item[field] && void ops.move(item, { [field]: value }, `Moved ${id}.`),
  });
  return [
    { id: 'edit', label: id, items: [{ id: 'edit', label: 'Edit', onSelect: () => onOpen(id) }] },
    {
      id: 'move',
      items: [
        moveRow('status', 'Move to status', STATUS_OPTIONS),
        moveRow('thread', 'Reassign to thread', threadChoices(threads, item.thread)),
      ],
    },
    {
      id: 'soft',
      items: [archive, { id: 'delete', label: 'Delete', tone: 'danger', onSelect: () => void ops.soft('delete', item) }, { id: 'copy', label: 'Copy id', onSelect: () => copyText(id) }],
    },
  ];
};

export { cardMenuGroups };
