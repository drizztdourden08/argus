/* @layer renderer-app @kind logic */
import { toMillis } from '../../format';
import type { ActivityItem } from './model.type';

const byTime = (a: ActivityItem, b: ActivityItem): number =>
  (toMillis(a.at) || 0) - (toMillis(b.at) || 0) || a.id.localeCompare(b.id, undefined, { numeric: true });

const sortActivity = (items: readonly ActivityItem[]): ActivityItem[] => [...items].sort(byTime);

export { sortActivity };
