/* @layer renderer-app @kind logic */
import { toMillis } from '../../../format';
import { MS_PER_MINUTE } from '../OverlordPresence.constants';

const ringProgress = (nextAt: unknown, everyMin: number | null | undefined, now: number): number | null => {
  const at = toMillis(nextAt);
  if (!Number.isFinite(at) || !everyMin || everyMin <= 0) return null;
  const left = (at - now) / (everyMin * MS_PER_MINUTE);
  return Math.min(1, Math.max(0, 1 - left));
};

export { ringProgress };
