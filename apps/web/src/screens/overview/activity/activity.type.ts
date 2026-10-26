/* @layer renderer-app @kind types */
import type { ActivityFilters, ActivityItem, LevelFilter } from './model';

interface ActivityTabProps {
  freshSince: number;
}

interface ActivityFilterBarProps {
  filters: ActivityFilters;
  levels: Record<LevelFilter, number>;
  kinds: readonly [string, number][];
  total: number;
  onChange: (next: ActivityFilters) => void;
}

interface ActivityFeedProps {
  items: readonly ActivityItem[];
  shown: number;
  freshSince: number;
  onMore: () => void;
}

interface ActivityRowProps {
  item: ActivityItem;
  freshSince: number;
}

export type { ActivityFeedProps, ActivityFilterBarProps, ActivityRowProps, ActivityTabProps };
