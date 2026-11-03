/* @layer renderer-app @kind logic */
import type { DepDoc } from '../../../../data/deps';
import { toMillis } from '../../../../format';
import { startedAt } from './started-at';

const waitMs = (doc: DepDoc, now: number): number => {
  const start = toMillis(startedAt(doc));
  if (!Number.isFinite(start)) return 0;
  const end = doc.resolvedAt ? toMillis(doc.resolvedAt) : now;
  return Math.max(0, (Number.isFinite(end) ? end : now) - start);
};

export { waitMs };
