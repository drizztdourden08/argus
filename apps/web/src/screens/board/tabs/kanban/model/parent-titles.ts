/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';

const parentTitles = (items: readonly ItemRef[]): Map<string, string> =>
  new Map(items.filter((item) => !item.deletedAt).map((item) => [item.id, item.title]));

export { parentTitles };
