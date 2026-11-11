/* @layer renderer-app @kind logic */
import type { ReviewRow } from '../../../../../data/types';
import { CHOICE_SKIP } from '../review.constants';
import type { Decisions } from '../review.type';
import { choiceOf } from './choice-of';

const applyCount = (rows: readonly ReviewRow[], decisions: Decisions, edits: number): number =>
  rows.filter((row) => row.diff && choiceOf(row, decisions) !== CHOICE_SKIP).length + edits;

export { applyCount };
