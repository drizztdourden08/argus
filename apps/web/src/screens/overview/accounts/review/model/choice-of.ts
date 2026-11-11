/* @layer renderer-app @kind logic */
import type { ReviewRow } from '../../../../../data/types';
import type { Decisions } from '../review.type';

const choiceOf = (row: ReviewRow, decisions: Decisions): string => decisions[row.id] ?? row.diff?.default ?? '';

export { choiceOf };
