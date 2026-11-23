/* @layer renderer-app @kind logic */
import type { KanbanColumnConfig } from '../../../../../data/types';

const targetsOf = (columns: readonly KanbanColumnConfig[] | null): ReadonlyMap<string, string> =>
  new Map((columns ?? []).flatMap((column) => (column.match.lanes[0] ? [[column.id, column.match.lanes[0]] as const] : [])));

export { targetsOf };
