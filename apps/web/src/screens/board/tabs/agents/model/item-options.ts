/* @layer renderer-app @kind logic */
import type { SelectOption } from '@drizztdourden08/tessera';
import type { Item } from '../../../../../data/kanban';
import type { WithId } from '../../../../../data/react';
import { ADD_ITEM_LABEL, ITEM_LABEL_CHARS, NO_ITEM } from './model.constants';

const itemOptions = (items: readonly WithId<Item>[], linked: readonly string[]): SelectOption[] => {
  const options = items
    .filter((item) => !item.deletedAt && !linked.includes(item.id))
    .map((item) => ({ value: item.id, label: `${item.id} ${String(item.title).slice(0, ITEM_LABEL_CHARS)}` }));
  return [{ value: NO_ITEM, label: ADD_ITEM_LABEL }, ...options];
};

export { itemOptions };
