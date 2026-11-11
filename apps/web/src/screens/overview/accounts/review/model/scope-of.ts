/* @layer renderer-app @kind logic */
import type { ReviewRow } from '../../../../../data/types';
import { NO_KIDS } from './scope-of.constants';

const scopeOf = (row: ReviewRow, scopes: ReadonlyMap<string, readonly ReviewRow[]>): ReviewRow[] => [row, ...(scopes.get(row.id) ?? NO_KIDS)];

export { scopeOf };
