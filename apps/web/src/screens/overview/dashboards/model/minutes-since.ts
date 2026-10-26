/* @layer renderer-app @kind logic */
import { toMillis } from '../../format';
import { MS_PER_MINUTE } from './model.constants';

const minutesSince = (iso: string | null | undefined, now: number): number => {
  const at = toMillis(iso);
  return iso && Number.isFinite(at) ? (now - at) / MS_PER_MINUTE : Number.POSITIVE_INFINITY;
};

export { minutesSince };
