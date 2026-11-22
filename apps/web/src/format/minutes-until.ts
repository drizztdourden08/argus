/* @layer renderer-app @kind logic */
import { toMillis } from './to-millis';

const MS_PER_MINUTE = 60_000;

const minutesUntil = (value: unknown, now: number): number | null => {
  const at = toMillis(value);
  return Number.isFinite(at) ? Math.max(0, Math.ceil((at - now) / MS_PER_MINUTE)) : null;
};

export { minutesUntil };
