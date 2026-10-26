/* @layer renderer-app @kind logic */
import { MINUTES_PER_HOUR, SECONDS_PER_MINUTE, TWO_DAYS_IN_HOURS, HOURS_PER_DAY } from './format.constants';
import { toMillis } from './to-millis';

const formatAgo = (value: unknown, now: number): string => {
  const at = toMillis(value);
  if (!Number.isFinite(at)) return 'never';
  const seconds = Math.max(0, Math.round((now - at) / 1000));
  if (seconds < SECONDS_PER_MINUTE) return `${seconds}s`;
  const minutes = Math.round(seconds / SECONDS_PER_MINUTE);
  if (minutes < MINUTES_PER_HOUR) return `${minutes}m`;
  const hours = Math.round(minutes / MINUTES_PER_HOUR);
  return hours < TWO_DAYS_IN_HOURS ? `${hours}h` : `${Math.round(hours / HOURS_PER_DAY)}d`;
};

export { formatAgo };
