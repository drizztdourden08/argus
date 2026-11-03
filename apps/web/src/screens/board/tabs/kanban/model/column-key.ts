/* @layer renderer-app @kind logic */
import { nonEmpty } from '../../../shared';
import { assigneeOf } from './assignee-of';
import { GENERAL, NO_PARENT } from './model.constants';
import type { CardData, ItemGroup } from './model.type';

const columnKey = (card: CardData, group: ItemGroup): string => {
  if (group === 'assignee') return nonEmpty(assigneeOf(card.item), GENERAL);
  if (group === 'parent') return nonEmpty(card.item.parent, NO_PARENT);
  return nonEmpty(card.item.thread, GENERAL);
};

export { columnKey };
