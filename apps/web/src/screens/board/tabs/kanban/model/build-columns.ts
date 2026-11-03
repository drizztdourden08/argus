/* @layer renderer-app @kind logic */
import type { KanbanColumn } from '@drizztdourden08/tessera';
import { columnKey } from './column-key';
import { columnOrder } from './column-order';
import { columnTitle } from './column-title';
import { CLOSED_STATUSES, STATUS_COLUMNS } from './model.constants';
import { sortCards } from './sort-cards';
import type { CardData, ColumnsInput } from './model.type';

const statusOf = (card: CardData): string => (card.item.status === 'dropped' ? 'done' : card.item.status);

const statusColumns = (input: ColumnsInput): KanbanColumn<CardData>[] =>
  STATUS_COLUMNS.filter((column) => input.showClosed || !CLOSED_STATUSES.includes(column.value)).map((column) => ({
    id: column.value,
    title: column.label,
    cards: sortCards(input.cards.filter((card) => statusOf(card) === column.value), input.order),
  }));

const keyedColumns = (input: ColumnsInput): KanbanColumn<CardData>[] => {
  const groups = new Map<string, CardData[]>(input.seeds.map((seed) => [seed, []]));
  for (const card of input.cards) {
    const key = columnKey(card, input.group);
    const list = groups.get(key);
    if (list) list.push(card);
    else groups.set(key, [card]);
  }
  return [...groups.keys()].sort(columnOrder(input.group)).map((key) => ({ id: key, title: columnTitle(key, input), cards: sortCards(groups.get(key) ?? [], input.order) }));
};

const buildColumns = (input: ColumnsInput): KanbanColumn<CardData>[] => (input.group === 'status' ? statusColumns(input) : keyedColumns(input));

export { buildColumns };
