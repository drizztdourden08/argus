/* @layer renderer-app @kind logic */
import type { ReviewRow } from '../../../../../data/types';
import type { Decisions, Pair, Side, SideState } from '../review.type';
import { choiceOf } from './choice-of';
import { sideChoice } from './side-choice';

const sideState = (rows: readonly ReviewRow[], decisions: Decisions, side: Side, pair: Pair): SideState => {
  const picks = rows.filter((row) => row.diff).map((row) => ({ row, id: row.diff ? sideChoice(row.diff, side, pair) : null }));
  const usable = picks.filter((pick) => pick.id !== null);
  return { enabled: usable.length > 0, active: usable.length > 0 && usable.length === picks.length && usable.every((pick) => choiceOf(pick.row, decisions) === pick.id) };
};

export { sideState };
