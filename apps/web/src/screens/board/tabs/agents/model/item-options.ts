/* @layer renderer-app @kind logic */
import type { SelectOption } from '@drizztdourden08/tessera';
import type { Item } from '../../../../../data/kanban';
import type { WithId } from '../../../../../data/react';
import { ITEM_LABEL_CHARS, NO_ITEM } from './model.constants';

const itemOptions = (items: readonly WithId<Item>[], current: string | null | undefined): SelectOption[] => {
  const options = items
    .filter((item) => !item.deletedAt)
    .map((item) => ({ value: item.id, label: `${item.id} ${String(item.title).slice(0, ITEM_LABEL_CHARS)}` }));
  if (current && !options.some((option) => option.value === current)) options.push({ value: current, label: current });
  return [{ value: NO_ITEM, label: 'none' }, ...options];
};

export { itemOptions };
