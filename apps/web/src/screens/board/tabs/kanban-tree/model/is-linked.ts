/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';
import { allowsParent } from '../../../../../data/registry';
import { typeOf } from '../../kanban/model';

const isLinked = (item: ItemRef, live: ReadonlyMap<string, ItemRef>): boolean => {
  const parent = item.parent ? live.get(item.parent) : undefined;
  return parent !== undefined && allowsParent(typeOf(item), typeOf(parent));
};

export { isLinked };
