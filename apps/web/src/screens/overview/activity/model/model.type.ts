/* @layer renderer-app @kind types */
import type { WithId } from '../../../../data/react';
import type { ActivityDoc, ActivityLevel } from '../../../../data/types';

type ActivityItem = WithId<ActivityDoc>;

type LevelFilter = 'all' | ActivityLevel;

interface RowContext {
  now: number;
  freshSince: number;
  nameOf: (key: string) => string;
}

export type { ActivityItem, LevelFilter, RowContext };
