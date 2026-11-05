/* @layer renderer-app @kind logic */
import type { SortDirection } from '@drizztdourden08/tessera';

const parseSort = <O extends string>(raw: string, orders: readonly O[], fallback: { order: O; direction: SortDirection }): { order: O; direction: SortDirection } => {
  const [order, direction] = raw.split(':');
  const known = orders.find((candidate) => candidate === order);
  if (!known || (direction !== 'asc' && direction !== 'desc')) return fallback;
  return { order: known, direction };
};

export { parseSort };
