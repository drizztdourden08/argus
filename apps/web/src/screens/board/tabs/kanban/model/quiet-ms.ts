/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';
import { MS_PER_HOUR, QUIET_HOURS } from './model.constants';

const quietMs = (item: ItemRef, waiting: boolean, now: number): number | null => {
  const hours = QUIET_HOURS[item.status];
  const changed = Date.parse(item.updatedAt ?? '');
  if (!hours || !Number.isFinite(changed) || item.archivedAt || waiting) return null;
  const elapsed = now - changed;
  return elapsed >= hours * MS_PER_HOUR ? elapsed : null;
};

export { quietMs };
