/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';
import type { CardProgress } from './model.type';

const acProgress = (item: ItemRef): CardProgress | null => {
  const criteria = item.ac ?? [];
  if (criteria.length === 0) return null;
  const met = criteria.filter((criterion) => criterion.done).length;
  return { met, total: criteria.length, text: `${met}/${criteria.length}`, title: `${met} of ${criteria.length} criteria met` };
};

export { acProgress };
