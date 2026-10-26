/* @layer renderer-app @kind logic */
import { MINUTES_PER_HOUR, SECONDS_PER_MINUTE } from './format.constants';

const pad = (value: number): string => String(value).padStart(2, '0');

const formatElapsed = (millis: number): string => {
  const seconds = Math.max(0, Math.floor(millis / 1000));
  if (!Number.isFinite(seconds)) return '';
  if (seconds < SECONDS_PER_MINUTE) return `${seconds}s`;
  const minutes = Math.floor(seconds / SECONDS_PER_MINUTE);
  if (minutes < MINUTES_PER_HOUR) return `${minutes}m ${pad(seconds % SECONDS_PER_MINUTE)}s`;
  return `${Math.floor(minutes / MINUTES_PER_HOUR)}h ${pad(minutes % MINUTES_PER_HOUR)}m`;
};

export { formatElapsed };
