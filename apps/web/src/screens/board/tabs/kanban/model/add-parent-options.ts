/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';
import { itemTypeOf } from '../../../../../data/registry';
import type { Option } from '../../../shared';
import { CLOSED_STATUSES } from './model.constants';
import { typeOf } from './type-of';

const addParentOptions = (type: string, items: ReadonlyMap<string, ItemRef>): Option[] => {
  const allowed = itemTypeOf(type)?.parents ?? [];
  return [...items.values()]
    .filter((item) => allowed.includes(typeOf(item)) && !CLOSED_STATUSES.includes(item.status))
    .map((item) => ({ value: item.id, label: `${item.id} ${item.title}` }));
};

export { addParentOptions };
