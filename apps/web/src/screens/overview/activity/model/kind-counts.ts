/* @layer renderer-app @kind logic */
import { kindOf } from './kind-of';
import { KNOWN_KINDS } from './model.constants';
import type { ActivityItem } from './model.type';

const kindCounts = (items: readonly ActivityItem[]): [string, number][] => {
  const counts = new Map<string, number>(KNOWN_KINDS.map((kind) => [kind, 0]));
  for (const item of items) counts.set(kindOf(item), (counts.get(kindOf(item)) ?? 0) + 1);
  return [...counts.entries()];
};

export { kindCounts };
