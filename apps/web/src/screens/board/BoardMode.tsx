/* @layer renderer-app @kind component */
import { useParams } from 'react-router';
import { usePortfolio } from '../../data/react';
import { NotFoundPage } from '../not-found/NotFoundPage';
import { BOARD_KEY_PATTERN } from './board.constants';
import { BoardStage } from './BoardStage';

const BoardMode = () => {
  const { key } = useParams();
  const { ready, cards } = usePortfolio();
  const missing = ready && cards.length > 0 && !cards.some((card) => card.key === key);
  if (!key || !BOARD_KEY_PATTERN.test(key) || missing) return <NotFoundPage message={key ? `There is no board for "${key}".` : undefined} />;
  return <BoardStage boardKey={key} />;
};

export { BoardMode };
