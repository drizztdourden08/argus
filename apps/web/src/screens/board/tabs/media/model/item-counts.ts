/* @layer renderer-app @kind logic */
import type { Shot } from '../../../../../data/media';
import type { ItemCount } from '../media.type';

const itemCounts = (shots: readonly Shot[]): ItemCount[] => {
  const counts = new Map<string, ItemCount>();
  for (const shot of shots) {
    const row = counts.get(shot.item) ?? { id: shot.item, title: shot.title, count: 0 };
    counts.set(shot.item, { ...row, count: row.count + 1 });
  }
  return [...counts.values()].sort((a, b) => a.id.localeCompare(b.id, undefined, { numeric: true }));
};

export { itemCounts };
