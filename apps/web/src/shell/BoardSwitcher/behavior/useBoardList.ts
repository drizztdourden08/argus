/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import type { Card } from '../../../data/types';
import { usePortfolio } from '../../../data/react';
import type { BoardOption } from '../BoardSwitcher.type';
import { boardOption } from './board-option';

interface BoardList {
  cards: Card[];
  options: BoardOption[];
}

const useBoardList = (): BoardList => {
  const { cards } = usePortfolio();
  const options = useMemo(() => cards.map(boardOption), [cards]);
  return { cards, options };
};

export { useBoardList };
