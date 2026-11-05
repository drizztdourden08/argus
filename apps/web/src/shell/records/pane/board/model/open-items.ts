/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';
import { LANES } from '../board.constants';
import type { LaneGroup } from '../board.type';

const isOpen = (item: ItemRef): boolean => !item.deletedAt && !item.archivedAt;

const openItems = (items: readonly ItemRef[]): LaneGroup[] =>
  LANES.map(({ lane, label }) => ({ lane, label, items: items.filter((item) => isOpen(item) && item.status === lane) })).filter((group) => group.items.length > 0);

export { openItems };
