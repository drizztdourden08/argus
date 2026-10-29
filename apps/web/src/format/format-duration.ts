/* @layer renderer-app @kind logic */
import { asNumber } from './as-number';
import { ABSENT, MS_PER_SECOND, SECONDS_PER_MINUTE, WHOLE_SECONDS_FROM } from './format.constants';

const formatDuration = (millis: unknown): string => {
  const value = asNumber(millis);
  if (value === null) return ABSENT;
  const seconds = value / MS_PER_SECOND;
  if (seconds < 1) return `${Math.round(value)}ms`;
  if (seconds < WHOLE_SECONDS_FROM) return `${seconds.toFixed(1)}s`;
  const whole = Math.round(seconds);
  if (whole < SECONDS_PER_MINUTE) return `${whole}s`;
  const rest = whole % SECONDS_PER_MINUTE;
  const minutes = Math.floor(whole / SECONDS_PER_MINUTE);
  return rest ? `${minutes}m ${rest}s` : `${minutes}m`;
};

export { formatDuration };
