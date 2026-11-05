/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { usePortfolio } from '../../../data/react';
import { boardOption } from './board-option';
import type { BoardList } from './board-list.type';

const useBoardList = (): BoardList => {
  const { cards } = usePortfolio();
  const options = useMemo(() => cards.map(boardOption), [cards]);
  return { cards, options };
};

export { useBoardList };
