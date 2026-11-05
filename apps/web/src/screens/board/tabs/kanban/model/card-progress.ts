/* @layer renderer-app @kind logic */
import { itemTypeOf } from '../../../../../data/registry';
import { runProgress } from './run-progress';
import { typeOf } from './type-of';
import type { CardData, CardProgress } from './model.type';

const cardProgress = (card: CardData): CardProgress[] => {
  if (itemTypeOf(typeOf(card.item))?.card.progress !== 'run') return [];
  const run = runProgress(card.item);
  return run ? [run] : [];
};

export { cardProgress };
