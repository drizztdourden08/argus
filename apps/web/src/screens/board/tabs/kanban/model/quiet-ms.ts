/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';
import { DEFAULT_QUIET_MS } from './quiet-limits.constants';

const quietMs = (item: ItemRef, waiting: boolean, now: number, limits: Readonly<Record<string, number>> = DEFAULT_QUIET_MS): number | null => {
  const limit = limits[item.status];
  const changed = Date.parse(item.updatedAt ?? '');
  if (!limit || !Number.isFinite(changed) || item.archivedAt || waiting) return null;
  const elapsed = now - changed;
  return elapsed >= limit ? elapsed : null;
};

export { quietMs };
