/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../../data/kanban';
import { CRUMB_MAX } from './pane-model.constants';

const crumbsOf = (item: ItemRef, items: readonly ItemRef[]): ItemRef[] => {
  const byId = new Map(items.map((other) => [other.id, other]));
  const crumbs: ItemRef[] = [];
  let parent = byId.get(item.parent ?? '');
  while (parent && parent.id !== item.id && !crumbs.includes(parent) && crumbs.length < CRUMB_MAX) {
    crumbs.unshift(parent);
    parent = byId.get(parent.parent ?? '');
  }
  return crumbs;
};

export { crumbsOf };
