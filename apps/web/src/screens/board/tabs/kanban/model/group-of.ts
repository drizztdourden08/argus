/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';
import { containerOf } from './container-of';
import type { ItemGroup } from './model.type';

const groupOf = (item: ItemRef, group: ItemGroup, shown: ReadonlyMap<string, ItemRef>): string | undefined =>
  group === 'parent' ? undefined : containerOf(item, shown);

export { groupOf };
