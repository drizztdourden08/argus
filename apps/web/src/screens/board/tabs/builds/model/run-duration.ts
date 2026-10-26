/* @layer renderer-app @kind logic */
import type { RunEntry } from '../../../../../data/runs';
import { formatDuration } from './format-duration';

const runDuration = (run: RunEntry): string => {
  const start = Date.parse(run.startedAt ?? run.queuedAt);
  const end = Date.parse(run.endedAt ?? '');
  return Number.isFinite(start) && Number.isFinite(end) ? formatDuration(end - start) : '';
};

export { runDuration };
