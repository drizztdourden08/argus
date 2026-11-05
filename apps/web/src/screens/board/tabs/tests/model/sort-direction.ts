/* @layer renderer-app @kind logic */
import type { SortDirection } from '@drizztdourden08/tessera';
import type { TestOrder } from './model.type';

const NATURAL_DIRECTION: Record<TestOrder, SortDirection> = { rank: 'asc', updated: 'desc' };

const directionOf = (raw: string | null | undefined): SortDirection | null => (raw === 'asc' || raw === 'desc' ? raw : null);

const shownDirection = (order: TestOrder, direction: SortDirection | null | undefined): SortDirection => direction ?? NATURAL_DIRECTION[order];

const isNatural = (order: TestOrder, direction: SortDirection | null | undefined): boolean => shownDirection(order, direction) === NATURAL_DIRECTION[order];

export { directionOf, isNatural, shownDirection };
