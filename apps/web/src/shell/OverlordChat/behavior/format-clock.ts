/* @layer renderer-app @kind logic */
import { MS_PER_SECOND, SECONDS_PER_MINUTE } from './format-clock.constants';

const formatClock = (millis: number): string => {
  const total = Math.round(millis / MS_PER_SECOND);
  const seconds = String(total % SECONDS_PER_MINUTE).padStart(2, '0');
  return `${Math.floor(total / SECONDS_PER_MINUTE)}:${seconds}`;
};

export { formatClock };
