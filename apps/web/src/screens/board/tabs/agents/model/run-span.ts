/* @layer renderer-app @kind logic */
import type { RunEntry } from '../../../../../data/runs';
import { MS_PER_SECOND, SECONDS_PER_HOUR, SECONDS_PER_MINUTE } from './model.constants';

const pad = (value: number): string => String(value).padStart(2, '0');

const spanText = (total: number): string => {
  if (!Number.isFinite(total) || total < 0) return '-';
  const hours = Math.floor(total / SECONDS_PER_HOUR);
  const minutes = Math.floor((total % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE);
  const seconds = total % SECONDS_PER_MINUTE;
  if (hours > 0) return `${hours}h ${pad(minutes)}m`;
  return minutes > 0 ? `${minutes}m ${pad(seconds)}s` : `${seconds}s`;
};

const runSpan = (run: RunEntry, now?: number): string => {
  const end = now ?? Date.parse(run.endedAt ?? '');
  return spanText(Math.floor((end - Date.parse(run.startedAt ?? run.queuedAt)) / MS_PER_SECOND));
};

export { runSpan };
