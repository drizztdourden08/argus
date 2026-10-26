/* @layer renderer-app @kind component */
import { useParams } from 'react-router';
import { NotFoundPage } from '../not-found/NotFoundPage';
import { BOARD_KEY_PATTERN } from './board.constants';
import { BoardPage } from './BoardPage';

const BoardRoute = () => {
  const { key } = useParams();
  if (!key || !BOARD_KEY_PATTERN.test(key)) return <NotFoundPage />;
  return <BoardPage boardKey={key} />;
};

export { BoardRoute };
