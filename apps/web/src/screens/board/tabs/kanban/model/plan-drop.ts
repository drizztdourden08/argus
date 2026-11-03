/* @layer renderer-app @kind logic */
import { moveCard } from '@drizztdourden08/tessera';
import type { KanbanMove } from '@drizztdourden08/tessera';
import type { PlaceAmong } from '../../../../../data/kanban';
import { placeAmong } from './place-among';
import type { DropInput, DropPlan, ItemOrder } from './model.type';

const withinColumn = (move: KanbanMove, place: PlaceAmong | null, order: ItemOrder): DropPlan => {
  if (!place || move.from.index === move.to.index) return { kind: 'none' };
  return order === 'rank' ? { kind: 'reorder', place } : { kind: 'order-off' };
};

const acrossColumns = (input: DropInput, place: PlaceAmong | null): DropPlan => {
  const { group, move, order } = input;
  if (group !== 'status' && group !== 'thread') return { kind: 'group-off' };
  return { kind: group, to: move.to.columnId, place: order === 'rank' && place ? place : {} };
};

const planDrop = (input: DropInput): DropPlan => {
  const { columns, move, order } = input;
  const landed = moveCard(columns, move).find((column) => column.id === move.to.columnId);
  const place = placeAmong(landed?.cards ?? [], move.cardId);
  return move.from.columnId === move.to.columnId ? withinColumn(move, place, order) : acrossColumns(input, place);
};

export { planDrop };
