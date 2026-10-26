/* @layer renderer-app @kind logic */
import type { KanbanCardData } from '@drizztdourden08/tessera';
import type { PlaceAmong } from '../../../../../data/kanban';

const placeAmong = (cards: readonly KanbanCardData[], id: string): PlaceAmong | null => {
  const index = cards.findIndex((card) => card.id === id);
  const next = cards[index + 1];
  const previous = cards[index - 1];
  if (next) return { before: next.id };
  if (previous) return { after: previous.id };
  return null;
};

export { placeAmong };
