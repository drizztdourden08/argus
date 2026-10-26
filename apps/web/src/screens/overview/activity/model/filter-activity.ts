/* @layer renderer-app @kind logic */
import { kindOf } from './kind-of';
import { levelOf } from './level-of';
import type { ActivityFilters, ActivityItem } from './model.type';

const filterActivity = (items: readonly ActivityItem[], filters: ActivityFilters): ActivityItem[] =>
  items.filter((item) => (filters.level === 'all' || levelOf(item) === filters.level) && (!filters.kind || kindOf(item) === filters.kind));

export { filterActivity };
