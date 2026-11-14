/* @layer renderer-app @kind logic */
import { toMillis } from '../../../../format';
import { MINUTES_PER_HOUR, MS_PER_SECOND, SECONDS_PER_MINUTE } from './model.constants';

const uptimeText = (startedAt: string | null, now: number): string => {
  const started = toMillis(startedAt);
  if (!Number.isFinite(started)) return '';
  const minutes = Math.max(0, Math.floor((now - started) / (MS_PER_SECOND * SECONDS_PER_MINUTE)));
  if (minutes < MINUTES_PER_HOUR) return `${minutes} min`;
  const rest = minutes % MINUTES_PER_HOUR;
  const hours = Math.floor(minutes / MINUTES_PER_HOUR);
  return rest ? `${hours} h ${rest} min` : `${hours} h`;
};

export { uptimeText };
