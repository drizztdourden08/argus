/* @layer renderer-app @kind logic */
import { moveCard } from '@drizztdourden08/tessera';
import { placeAmong } from './place-among';
import type { DropInput, DropPlan } from './model.type';

const planDrop = (input: DropInput): DropPlan => {
  const { columns, move, group, order } = input;
  const landed = moveCard(columns, move).find((column) => column.id === move.to.columnId);
  const place = placeAmong(landed?.cards ?? [], move.cardId);
  const manual = order === 'rank';
  if (move.from.columnId === move.to.columnId) {
    if (!place || move.from.index === move.to.index) return { kind: 'none' };
    return manual ? { kind: 'reorder', place } : { kind: 'order-off' };
  }
  return { kind: group, to: move.to.columnId, place: manual && place ? place : {} };
};

export { planDrop };
