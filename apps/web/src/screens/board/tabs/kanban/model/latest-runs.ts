/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';

const numberOf = (id: string): number => Number(id.replace(/^\D+/, ''));

const planOf = (item: ItemRef): string | null => (typeof item.fields?.plan === 'string' ? item.fields.plan : null);

const latestRuns = (items: readonly ItemRef[]): Map<string, ItemRef> => {
  const latest = new Map<string, ItemRef>();
  for (const item of items) {
    const plan = planOf(item);
    if (item.type !== 'test-run' || !plan || item.deletedAt || item.stage === 'abandoned') continue;
    const last = latest.get(plan);
    if (!last || numberOf(item.id) > numberOf(last.id)) latest.set(plan, item);
  }
  return latest;
};

export { latestRuns };
