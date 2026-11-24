/* @layer renderer-app @kind logic */
import { HOURS_PER_DAY, MINUTES_PER_HOUR, SECONDS_PER_MINUTE, TWO_DAYS_IN_HOURS } from './format.constants';
import { toMillis } from './to-millis';

const unit = (count: number, one: string, many: string): string => `${count} ${count === 1 ? one : many}`;

const agoWords = (value: unknown, now: number): string => {
  const at = toMillis(value);
  if (!Number.isFinite(at)) return 'never';
  const seconds = Math.max(0, Math.round((now - at) / 1000));
  if (seconds < SECONDS_PER_MINUTE) return `${unit(seconds, 'second', 'seconds')} ago`;
  const minutes = Math.round(seconds / SECONDS_PER_MINUTE);
  if (minutes < MINUTES_PER_HOUR) return `${minutes} min ago`;
  const hours = Math.round(minutes / MINUTES_PER_HOUR);
  if (hours < TWO_DAYS_IN_HOURS) return `${unit(hours, 'hour', 'hours')} ago`;
  return `${Math.round(hours / HOURS_PER_DAY)} days ago`;
};

export { agoWords };
