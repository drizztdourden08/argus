/* @layer renderer-app @kind logic */
import type { ReviewDiff } from '../../../../../data/types';
import { CHOICE_SKIP } from '../review.constants';
import type { Pair } from '../review.type';
import { sideWinner } from './side-winner';

const otherChoices = (diff: ReviewDiff, pair: Pair): string[] => diff.choices.filter((choice) => choice.id !== CHOICE_SKIP && sideWinner(choice.id, pair) === null).map((choice) => choice.id);

export { otherChoices };
