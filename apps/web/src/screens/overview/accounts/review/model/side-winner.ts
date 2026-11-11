/* @layer renderer-app @kind logic */
import type { Pair, Side } from '../review.type';
import { SIDE_ROLES } from './side-choice.constants';

const sideWinner = (choice: string, pair: Pair): Side | null => {
  const [verb = '', target] = choice.split(':');
  const role = SIDE_ROLES[verb];
  if (!role || (target !== pair.left && target !== pair.right)) return null;
  return (target === pair.left) === (role === 'winner') ? 'left' : 'right';
};

export { sideWinner };
