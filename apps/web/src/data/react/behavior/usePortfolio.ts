/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import type { Card } from '../../types';
import { PORTFOLIO_SCOPE } from '../react.constants';
import type { PortfolioView } from '../react.type';
import { useCollection } from './useCollection';
import { useStreamStatus } from './useStreamStatus';

const byName = (a: Card, b: Card): number => a.name.localeCompare(b.name);

const usePortfolio = (): PortfolioView => {
  const cards = useCollection<Card>(PORTFOLIO_SCOPE, 'cards');
  const status = useStreamStatus(PORTFOLIO_SCOPE);
  const sorted = useMemo(() => [...cards.docs].sort(byName), [cards.docs]);
  return { ready: cards.ready, status, cards: sorted };
};

export { usePortfolio };
