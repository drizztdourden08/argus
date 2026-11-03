/* @layer renderer-app @kind logic */
import type { MenuGroup, MenuItem } from '@drizztdourden08/tessera';
import { recordMenuGroups } from '../../../../../shell';
import { copyText } from '../../../shared';
import type { ItemDoc } from '../model';
import { CASE_KIND } from '../tests.constants';
import type { TestMenuInput } from '../tests.type';

const MOVE_LABEL_CHARS = 40;

const moveTarget = (item: ItemDoc, current: string | null, move: (itemId: string) => void): MenuItem => ({
  id: `item-${item.id}`,
  label: `${item.id} ${item.title.slice(0, MOVE_LABEL_CHARS)}`,
  kind: 'radio',
  checked: item.id === current,
  onSelect: () => {
    if (item.id !== current) move(item.id);
  },
});

const testMenuGroups = (input: TestMenuInput): MenuGroup[] => {
  const { test, shared, rerun } = input;
  const { id } = test;
  const { ops, items } = shared;
  const archive: MenuItem = test.archivedAt
    ? { id: 'unarchive', label: 'Unarchive', onSelect: () => void ops.soft('unarchive', test) }
    : { id: 'archive', label: 'Archive', onSelect: () => void ops.soft('archive', test) };
  const targets = items.filter((item) => !item.deletedAt).map((item) => moveTarget(item, test.item, (itemId) => void ops.move(test, itemId)));
  const again: MenuItem[] = test.status === 'pending' ? [] : [{ id: 'rerun', label: 'Run again', onSelect: rerun }];
  const middle: MenuGroup[] = [
    { id: 'run', items: [{ id: 'note', label: 'Add a note...', onSelect: () => shared.onNote(id) }, ...again] },
    { id: 'edit', items: [{ id: 'edit', label: 'Edit', onSelect: () => shared.onEdit(id) }] },
    { id: 'move', items: [{ id: 'item', label: 'Move to item', children: targets }] },
    { id: 'tidy', items: [archive, { id: 'delete', label: 'Delete', tone: 'danger', onSelect: () => void ops.soft('delete', test) }] },
  ];
  return recordMenuGroups(CASE_KIND, id, { open: () => shared.onOpen(id), copy: () => copyText(id) }, middle);
};

export { testMenuGroups };
