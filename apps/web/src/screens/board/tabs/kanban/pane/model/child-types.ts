/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../../data/kanban';
import { ITEM_REGISTRY } from '../../../../../../data/registry';
import type { ItemType } from '../../../../../../data/registry';
import { typeOf } from '../../model';

const childTypes = (item: ItemRef): ItemType[] =>
  ITEM_REGISTRY.types.filter((type) => type.collection === 'items' && type.parents.includes(typeOf(item)));

export { childTypes };
