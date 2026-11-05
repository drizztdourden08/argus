/* @layer renderer-app @kind logic */
import type { SortDirection } from '@drizztdourden08/tessera';

const directionOf = (raw: string | null | undefined): SortDirection | null => (raw === 'asc' || raw === 'desc' ? raw : null);

export { directionOf };
