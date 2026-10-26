/* @layer renderer-app @kind logic */
import { asNumber } from './as-number';
import { ABSENT, HOURS_PER_DAY, MINUTES_PER_HOUR, TWO_DAYS_IN_HOURS } from './format.constants';

const formatMinutes = (minutes: unknown): string => {
  const value = asNumber(minutes);
  if (value === null) return ABSENT;
  if (value < 1) return '<1m';
  if (value < MINUTES_PER_HOUR) return `${Math.round(value)}m`;
  const hours = Math.floor(value / MINUTES_PER_HOUR);
  const rest = Math.round(value % MINUTES_PER_HOUR);
  if (hours >= TWO_DAYS_IN_HOURS) return `${Math.floor(hours / HOURS_PER_DAY)}d`;
  return rest ? `${hours}h ${rest}m` : `${hours}h`;
};

export { formatMinutes };
