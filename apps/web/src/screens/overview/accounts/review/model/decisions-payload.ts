/* @layer renderer-app @kind logic */
import type { ReviewRow } from '../../../../../data/types';
import type { Decisions } from '../review.type';
import { choiceOf } from './choice-of';

const decisionsPayload = (rows: readonly ReviewRow[], decisions: Decisions): Decisions =>
  Object.fromEntries(rows.filter((row) => row.diff).map((row) => [row.id, choiceOf(row, decisions)]));

export { decisionsPayload };
