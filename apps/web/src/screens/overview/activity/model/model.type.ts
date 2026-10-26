/* @layer renderer-app @kind types */
import type { WithId } from '../../../../data/react';
import type { ActivityDoc, ActivityLevel } from '../../../../data/types';

type ActivityItem = WithId<ActivityDoc>;

type LevelFilter = 'all' | ActivityLevel;

interface ActivityFilters {
  level: LevelFilter;
  kind: string;
}

export type { ActivityFilters, ActivityItem, LevelFilter };
