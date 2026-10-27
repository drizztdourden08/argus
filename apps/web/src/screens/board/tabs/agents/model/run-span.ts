/* @layer renderer-app @kind logic */
import type { RunEntry } from '../../../../../data/runs';
import { SECONDS_PER_HOUR, SECONDS_PER_MINUTE } from './model.constants';

const pad = (value: number): string => String(value).padStart(2, '0');

const runSpan = (run: RunEntry): string => {
  const total = Math.round((Date.parse(run.endedAt ?? '') - Date.parse(run.startedAt ?? run.queuedAt)) / 1000);
  if (!Number.isFinite(total) || total < 0) return '-';
  const hours = Math.floor(total / SECONDS_PER_HOUR);
  const minutes = Math.floor((total % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE);
  const seconds = total % SECONDS_PER_MINUTE;
  if (hours > 0) return `${hours}h ${pad(minutes)}m`;
  return minutes > 0 ? `${minutes}m ${pad(seconds)}s` : `${seconds}s`;
};

export { runSpan };
