/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { PORTFOLIO_SCOPE, useCollection, usePortfolio } from '../../react';
import { BOARDS_COLLECTION } from '../boards.constants';
import type { BoardDoc, BoardItems } from '../boards.type';
import { joinBoards } from '../join-boards';

const useBoardItems = (): BoardItems => {
  const boards = useCollection<BoardDoc>(PORTFOLIO_SCOPE, BOARDS_COLLECTION);
  const portfolio = usePortfolio();
  const items = useMemo(() => joinBoards(boards.docs, portfolio.cards), [boards.docs, portfolio.cards]);
  return { ready: boards.ready && portfolio.ready, items };
};

export { useBoardItems };
