/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';
import type { ItemGroup } from './model.type';

const groupOf = (item: ItemRef, group: ItemGroup, parents: ReadonlyMap<string, ItemRef>): string | undefined =>
  group !== 'parent' && item.parent && parents.has(item.parent) ? item.parent : undefined;

export { groupOf };
