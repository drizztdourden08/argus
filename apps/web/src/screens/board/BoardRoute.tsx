/* @layer renderer-app @kind component */
import { useParams } from 'react-router';
import { usePortfolio } from '../../data/react';
import { NotFoundPage } from '../not-found/NotFoundPage';
import { BOARD_KEY_PATTERN } from './board.constants';
import { BoardPage } from './BoardPage';

const BoardRoute = () => {
  const { key } = useParams();
  const portfolio = usePortfolio();
  const unknown = portfolio.ready && !portfolio.cards.some((card) => card.key === key);
  if (!key || !BOARD_KEY_PATTERN.test(key) || unknown) return <NotFoundPage message={key ? `There is no board for "${key}".` : undefined} />;
  return <BoardPage boardKey={key} />;
};

export { BoardRoute };
