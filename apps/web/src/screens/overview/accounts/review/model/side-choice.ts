/* @layer renderer-app @kind logic */
import type { ReviewDiff } from '../../../../../data/types';
import { CHOICE_SKIP } from '../review.constants';
import type { Pair, Side } from '../review.type';
import { sideWinner } from './side-winner';

const sideChoice = (diff: ReviewDiff, side: Side, pair: Pair): string | null => {
  if (side === 'skip') return diff.choices.some((choice) => choice.id === CHOICE_SKIP) ? CHOICE_SKIP : null;
  return diff.choices.find((choice) => sideWinner(choice.id, pair) === side)?.id ?? null;
};

export { sideChoice };
