/* @layer renderer-app @kind logic */
import { HOURS_AS_DAYS_FROM, HOURS_PER_DAY, MINUTES_PER_HOUR, MS_PER_MINUTE } from './model.constants';

const formatDuration = (ms: number): string => {
  const minutes = Math.floor(Math.max(0, ms) / MS_PER_MINUTE);
  if (minutes < 1) return '<1m';
  if (minutes < MINUTES_PER_HOUR) return `${minutes}m`;
  const hours = Math.floor(minutes / MINUTES_PER_HOUR);
  if (hours >= HOURS_AS_DAYS_FROM) return `${Math.floor(hours / HOURS_PER_DAY)}d`;
  const rest = minutes % MINUTES_PER_HOUR;
  return rest ? `${hours}h ${rest}m` : `${hours}h`;
};

export { formatDuration };
