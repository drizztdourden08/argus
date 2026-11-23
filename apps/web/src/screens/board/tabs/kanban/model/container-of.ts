/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';

const containerOf = (item: ItemRef, shown: ReadonlyMap<string, ItemRef>): string | undefined => {
  const seen = new Set<string>([item.id]);
  let root: ItemRef = item;
  for (let parent = shown.get(root.parent ?? ''); parent && !seen.has(parent.id); parent = shown.get(root.parent ?? '')) {
    seen.add(parent.id);
    root = parent;
  }
  return root.id === item.id ? undefined : root.id;
};

export { containerOf };
