/* @layer renderer-app @kind logic */
import { HOUR, MINUTE, SECOND } from './format-duration.constants';

const pad = (value: number): string => String(value).padStart(2, '0');

const formatDuration = (ms: number): string => {
  const seconds = Math.max(0, Math.floor(ms / SECOND));
  if (seconds < MINUTE) return `${seconds}s`;
  const minutes = Math.floor(seconds / MINUTE);
  if (minutes < HOUR) return `${minutes}m ${pad(seconds % MINUTE)}s`;
  return `${Math.floor(minutes / HOUR)}h ${pad(minutes % HOUR)}m`;
};

export { formatDuration };
