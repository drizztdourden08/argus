/* @layer renderer-app @kind logic */
import { NO_PARENT, OWNER } from './model.constants';
import type { ItemGroup } from './model.type';

const rankOf = (key: string, group: ItemGroup): number => {
  if (group === 'assignee' && key === OWNER) return 0;
  return group === 'parent' && key === NO_PARENT ? 2 : 1;
};

const columnOrder = (group: ItemGroup) => (a: string, b: string): number =>
  rankOf(a, group) - rankOf(b, group) || a.localeCompare(b, undefined, { numeric: true });

export { columnOrder };
