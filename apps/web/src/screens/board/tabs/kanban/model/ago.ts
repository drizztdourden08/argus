/* @layer renderer-app @kind logic */
import { HOURS_AS_DAYS_FROM, HOURS_PER_DAY, MINUTES_PER_HOUR, MS_PER_SECOND, SECONDS_PER_MINUTE } from './model.constants';

const ago = (iso: string | null | undefined, now: number): string => {
  if (!iso) return 'never';
  const seconds = Math.max(0, Math.round((now - Date.parse(iso)) / MS_PER_SECOND));
  if (seconds < SECONDS_PER_MINUTE) return `${seconds}s`;
  const minutes = Math.round(seconds / SECONDS_PER_MINUTE);
  if (minutes < MINUTES_PER_HOUR) return `${minutes}m`;
  const hours = Math.round(minutes / MINUTES_PER_HOUR);
  return hours < HOURS_AS_DAYS_FROM ? `${hours}h` : `${Math.round(hours / HOURS_PER_DAY)}d`;
};

export { ago };
