/* @layer renderer-app @kind logic */
import type { Card } from '../../../../data/types';
import { EMPTY_COUNTS } from './model.constants';
import type { CardCountsView } from './model.type';

const cardCounts = (card: Card): CardCountsView => {
  const counts = { ...EMPTY_COUNTS, ...card.counts };
  return {
    agents: counts.agents,
    items: counts.itemsOpen,
    notPassing: Math.max(0, counts.tests - counts.testsPass),
    failing: counts.testsFailing,
  };
};

export { cardCounts };
