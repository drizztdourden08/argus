/* @layer renderer-app @kind types */
import type { LevelFilter } from './model';

interface ActivityTabProps {
  freshSince: number;
}

interface LevelFilterProps {
  value: LevelFilter;
  counts: Record<LevelFilter, number>;
  onChange: (value: LevelFilter) => void;
}

export type { ActivityTabProps, LevelFilterProps };
