/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../data/kanban';
import type { Option } from '../../../../screens/board/shared';
import { NO_PARENT, NO_PARENT_TITLE, typeOf } from '../../../../screens/board/tabs/kanban/model';
import { paneType } from './pane-type';

const below = (items: readonly ItemRef[], id: string): Set<string> => {
  const found = new Set([id]);
  let grew = true;
  while (grew) {
    const size = found.size;
    items.filter((item) => item.parent && found.has(item.parent)).forEach((item) => found.add(item.id));
    grew = found.size > size;
  }
  return found;
};

const parentChoices = (item: ItemRef, items: readonly ItemRef[]): Option[] => {
  const type = paneType(item);
  if (!type || type.parents.length === 0) return [];
  const excluded = below(items, item.id);
  const fits = items.filter((other) => !excluded.has(other.id) && !other.deletedAt && type.parents.includes(typeOf(other)));
  return [{ value: NO_PARENT, label: NO_PARENT_TITLE }, ...fits.map((other) => ({ value: other.id, label: `${other.id} ${other.title}` }))];
};

export { parentChoices };
