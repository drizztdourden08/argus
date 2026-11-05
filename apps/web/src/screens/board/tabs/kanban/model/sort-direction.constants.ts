/* @layer renderer-app @kind constants */
import type { SortDirection } from '@drizztdourden08/tessera';
import type { ItemOrder } from './model.type';

const NATURAL_DIRECTION: Record<ItemOrder, SortDirection> = { rank: 'asc', updated: 'desc' };

export { NATURAL_DIRECTION };
