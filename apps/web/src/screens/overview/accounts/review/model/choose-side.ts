/* @layer renderer-app @kind logic */
import type { ChooseSideArgs } from '../review.type';
import { sideChoice } from './side-choice';

const chooseSide = (args: ChooseSideArgs): void => {
  const { rows, side, pair, revert, onChoose } = args;
  for (const row of rows) {
    if (!row.diff) continue;
    const id = sideChoice(row.diff, side, pair);
    if (id) onChoose(row.id, revert ? row.diff.default : id, row.diff.default);
  }
};

export { chooseSide };
