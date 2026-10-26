/* @layer renderer-app @kind logic */
import type { MenuGroup, MenuItem } from '@drizztdourden08/tessera';
import type { CardMenuProps } from '../kanban-props.type';
import { copyText, threadChoices } from '../../../shared';
import { STATUS_OPTIONS } from '../model';

const choice = (id: string, label: string, checked: boolean, pick: () => void): MenuItem => ({
  id,
  label,
  kind: 'radio',
  checked,
  onSelect: () => {
    if (!checked) pick();
  },
});

const cardMenuGroups = (props: CardMenuProps): MenuGroup[] => {
  const { item, threads, ops, onOpen } = props;
  const { id } = item;
  const archive: MenuItem = item.archivedAt
    ? { id: 'unarchive', label: 'Unarchive', onSelect: () => void ops.soft('unarchive', item) }
    : { id: 'archive', label: 'Archive', onSelect: () => void ops.soft('archive', item) };
  const statuses = STATUS_OPTIONS.map((option) =>
    choice(`status-${option.value}`, option.label, option.value === item.status, () => void ops.move(item, { status: option.value }, `Moved ${id}.`)));
  const owners = threadChoices(threads, item.thread).map((option) =>
    choice(`thread-${option.value}`, option.label, option.value === item.thread, () => void ops.move(item, { thread: option.value }, `Moved ${id}.`)));
  return [
    { id: 'open', items: [{ id: 'open', label: 'Open details', onSelect: () => onOpen(id) }, { id: 'copy', label: 'Copy id', onSelect: () => copyText(id) }] },
    { id: 'move', items: [{ id: 'status', label: 'Move to status', children: statuses }, { id: 'thread', label: 'Reassign to thread', children: owners }] },
    { id: 'soft', items: [archive, { id: 'delete', label: 'Delete', tone: 'danger', onSelect: () => void ops.soft('delete', item) }] },
  ];
};

export { cardMenuGroups };
