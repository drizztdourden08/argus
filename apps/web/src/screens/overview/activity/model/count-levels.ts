/* @layer renderer-app @kind logic */
import { levelOf } from './level-of';
import type { ActivityItem, LevelFilter } from './model.type';

const countLevels = (items: readonly ActivityItem[]): Record<LevelFilter, number> => {
  const counts: Record<LevelFilter, number> = { all: items.length, info: 0, warn: 0, error: 0 };
  for (const item of items) counts[levelOf(item)] += 1;
  return counts;
};

export { countLevels };
