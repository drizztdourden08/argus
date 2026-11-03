/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../../data/kanban';

const rankOf = (item: ItemRef): number => item.rank ?? 0;

const childrenOf = (items: readonly ItemRef[], parent: string): ItemRef[] =>
  items.filter((item) => item.parent === parent && !item.deletedAt && !item.archivedAt).sort((a, b) => rankOf(a) - rankOf(b));

export { childrenOf };
