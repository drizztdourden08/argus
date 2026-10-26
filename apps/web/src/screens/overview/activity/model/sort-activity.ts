/* @layer renderer-app @kind logic */
import { toMillis } from '../../../../format';
import type { ActivityItem } from './model.type';

const newestFirst = (a: ActivityItem, b: ActivityItem): number =>
  (toMillis(b.at) || 0) - (toMillis(a.at) || 0) || b.id.localeCompare(a.id, undefined, { numeric: true });

const sortActivity = (items: readonly ActivityItem[]): ActivityItem[] => [...items].sort(newestFirst);

export { sortActivity };
