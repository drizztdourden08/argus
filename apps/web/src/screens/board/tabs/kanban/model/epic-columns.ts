/* @layer renderer-app @kind logic */
import type { KanbanColumn } from '@drizztdourden08/tessera';
import type { ItemRef } from '../../../../../data/kanban';
import { clusterGroups } from './cluster-groups';
import { EPIC_KEY_PREFIX, NO_EPIC_ID } from './epic.constants';
import type { CardData } from './model.type';
import { typeOf } from './type-of';

const epicOf = (card: CardData, parents: ReadonlyMap<string, ItemRef>): string => {
  if (card.group) return card.group;
  const parent = card.item.parent;
  return parent && parents.has(parent) ? parent : NO_EPIC_ID;
};

const epicColumns = (columns: readonly KanbanColumn<CardData>[], parents: ReadonlyMap<string, ItemRef>): KanbanColumn<CardData>[] =>
  columns.map((column) => ({
    ...column,
    cards: clusterGroups(
      column.cards.map((card) => (typeOf(card.item) === 'epic' && !card.group ? card : { ...card, group: `${EPIC_KEY_PREFIX}${epicOf(card, parents)}:${column.id}` })),
    ),
  }));

export { epicColumns };
