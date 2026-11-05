/* @layer renderer-app @kind logic */
import type { Card } from '../../../data/types';

const boardsTarget = (last: string | null, cards: readonly Card[]): string | null => {
  if (last && (cards.length === 0 || cards.some((card) => card.key === last))) return last;
  return cards[0]?.key ?? null;
};

export { boardsTarget };
