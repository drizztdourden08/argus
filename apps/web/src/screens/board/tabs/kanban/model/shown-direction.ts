/* @layer renderer-app @kind logic */
import type { SortDirection } from '@drizztdourden08/tessera';
import type { ItemOrder } from './model.type';
import { NATURAL_DIRECTION } from './sort-direction.constants';

const shownDirection = (order: ItemOrder, direction: SortDirection | null | undefined): SortDirection => direction ?? NATURAL_DIRECTION[order];

export { shownDirection };
