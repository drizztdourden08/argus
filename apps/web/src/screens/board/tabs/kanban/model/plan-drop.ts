/* @layer renderer-app @kind logic */
import { moveCard } from '@drizztdourden08/tessera';
import type { KanbanMove } from '@drizztdourden08/tessera';
import type { PlaceAmong } from '../../../../../data/kanban';
import { placeAmong } from './place-among';
import type { DropInput, DropPlan } from './model.type';
import { isNatural } from './sort-direction';

const manual = (input: DropInput): boolean => input.order === 'rank' && isNatural(input.order, input.direction);

const withinColumn = (input: DropInput, place: PlaceAmong | null): DropPlan => {
  if (!place || input.move.from.index === input.move.to.index) return { kind: 'none' };
  return manual(input) ? { kind: 'reorder', place } : { kind: 'order-off' };
};

const acrossColumns = (input: DropInput, place: PlaceAmong | null): DropPlan => {
  const { group, move } = input;
  if (group !== 'status' && group !== 'thread') return { kind: 'group-off' };
  return { kind: group, to: move.to.columnId, place: manual(input) && place ? place : {} };
};

const planDrop = (input: DropInput): DropPlan => {
  const { columns, move } = input;
  const landed = moveCard(columns, move).find((column) => column.id === move.to.columnId);
  const place = placeAmong(landed?.cards ?? [], move.cardId);
  return move.from.columnId === move.to.columnId ? withinColumn(input, place) : acrossColumns(input, place);
};

export { planDrop };
