/* @layer renderer-app @kind logic */
import type { KanbanColumnConfig } from '../../../../../data/types';
import type { ColumnLimit } from './model.type';

const limitsOf = (columns: readonly KanbanColumnConfig[] | null): ReadonlyMap<string, ColumnLimit> =>
  new Map((columns ?? []).flatMap((column) => (column.limit ? [[column.id, column.limit] as const] : [])));

export { limitsOf };
