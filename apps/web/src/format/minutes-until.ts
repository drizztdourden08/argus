/* @layer renderer-app @kind logic */
import { MS_PER_MINUTE } from './format.constants';
import { toMillis } from './to-millis';

const minutesUntil = (value: unknown, now: number): number | null => {
  const at = toMillis(value);
  return Number.isFinite(at) ? Math.max(0, Math.ceil((at - now) / MS_PER_MINUTE)) : null;
};

export { minutesUntil };
