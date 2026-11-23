/* @layer renderer-app @kind logic */
import type { KanbanColumn } from '@drizztdourden08/tessera';
import { epicKeyOf } from './epic-key';
import type { CardData } from './model.type';

const epicKeys = (columns: readonly KanbanColumn<CardData>[]): string[] => {
  const keys = columns.flatMap((column) => column.cards.flatMap((card) => (card.group && epicKeyOf(card.group) ? [card.group] : [])));
  return [...new Set(keys)];
};

export { epicKeys };
