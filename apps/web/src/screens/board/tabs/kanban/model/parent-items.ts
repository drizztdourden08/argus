/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';

const parentItems = (items: readonly ItemRef[]): Map<string, ItemRef> => new Map(items.filter((item) => !item.deletedAt).map((item) => [item.id, item]));

export { parentItems };
