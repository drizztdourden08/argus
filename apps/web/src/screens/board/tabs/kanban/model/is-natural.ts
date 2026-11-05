/* @layer renderer-app @kind logic */
import type { SortDirection } from '@drizztdourden08/tessera';
import type { ItemOrder } from './model.type';
import { shownDirection } from './shown-direction';
import { NATURAL_DIRECTION } from './sort-direction.constants';

const isNatural = (order: ItemOrder, direction: SortDirection | null | undefined): boolean => shownDirection(order, direction) === NATURAL_DIRECTION[order];

export { isNatural };
