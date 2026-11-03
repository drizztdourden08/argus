/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../../../data/kanban';

const bundleOf = (item: ItemRef): string | null => {
  const bundle = item.fields?.bundle;
  return typeof bundle === 'string' ? bundle : null;
};

export { bundleOf };
