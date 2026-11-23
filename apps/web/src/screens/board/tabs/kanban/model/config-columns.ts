/* @layer renderer-app @kind logic */
import type { KanbanColumn, TagCategoryColor } from '@drizztdourden08/tessera';
import type { ItemRef } from '../../../../../data/kanban';
import type { KanbanColumnConfig, KanbanConfig } from '../../../../../data/types';
import { clusterGroups } from './cluster-groups';
import { ACCENTS, CLOSED_STATUSES } from './model.constants';
import { sortCards } from './sort-cards';
import type { CardData, ColumnsInput } from './model.type';

const columnsOf = (config: KanbanConfig | null | undefined): KanbanColumnConfig[] | null =>
  config && Array.isArray(config.columns) && config.columns.length > 0 ? config.columns : null;

const takes = (column: KanbanColumnConfig, item: ItemRef): boolean => {
  const stages = column.match.stages ?? [];
  return column.match.lanes.includes(item.status) && (stages.length === 0 || stages.includes(item.stage ?? ''));
};

const columnOf = (columns: readonly KanbanColumnConfig[], item: ItemRef): KanbanColumnConfig | undefined => columns.find((column) => takes(column, item));

const accentOf = (column: KanbanColumnConfig): TagCategoryColor | undefined => ACCENTS.find((accent) => accent === column.accent);

const isClosedColumn = (column: KanbanColumnConfig): boolean => column.match.lanes.every((lane) => CLOSED_STATUSES.includes(lane));

const configColumns = (input: ColumnsInput, columns: readonly KanbanColumnConfig[]): KanbanColumn<CardData>[] =>
  columns
    .filter((column) => input.showClosed || !isClosedColumn(column))
    .map((column) => ({
      id: column.id,
      title: column.name,
      accent: accentOf(column),
      cards: clusterGroups(sortCards(input.cards.filter((card) => columnOf(columns, card.item)?.id === column.id), input.order, input.direction)),
    }));

export { columnOf, columnsOf, configColumns };
