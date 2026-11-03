/* @layer renderer-app @kind logic */
import type { MenuGroup, MenuItem } from '@drizztdourden08/tessera';
import { RECORD_ACTIONS, recordMenuGroups } from '../../../../../shell';
import { copyText, threadChoices } from '../../../shared';
import type { Option } from '../../../shared';
import type { CardMenuProps } from '../kanban-props.type';
import { DEFAULT_TYPE, STATUS_OPTIONS, typeOf } from '../model';

const cardMenuGroups = (props: CardMenuProps): MenuGroup[] => {
  const { item, threads, ops, onOpen } = props;
  const { id } = item;
  const kind = typeOf(item);
  const moveRow = (field: 'status' | 'thread', label: string, options: readonly Option[]): MenuItem => ({
    id: field,
    label,
    kind: 'select',
    options,
    selected: item[field],
    onPick: (value) => value !== item[field] && void ops.move(item, { [field]: value }, `Moved ${id}.`),
  });
  const move: MenuGroup = {
    id: 'move',
    items: [moveRow('status', 'Move to status', STATUS_OPTIONS), moveRow('thread', 'Reassign to thread', threadChoices(threads, item.thread))],
  };
  const soft = (kindOfSoft: 'archive' | 'unarchive' | 'delete') => () => void ops.soft(kindOfSoft, item);
  const handlers = {
    open: () => onOpen(id),
    [item.archivedAt ? 'unarchive' : 'archive']: soft(item.archivedAt ? 'unarchive' : 'archive'),
    delete: soft('delete'),
    copy: () => copyText(id),
  };
  return recordMenuGroups(RECORD_ACTIONS[kind] ? kind : DEFAULT_TYPE, id, handlers, [move]);
};

export { cardMenuGroups };
