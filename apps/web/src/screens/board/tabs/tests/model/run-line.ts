/* @layer renderer-app @kind logic */
import { END_TONES, LAUNCHING_LABEL, RUNNING_LABEL, QUEUED_LABEL } from './model.constants';
import type { RunEntry, RunLine } from './model.type';
import { plainLine } from './plain-line';

const endLabel = (run: RunEntry): string => {
  if (run.status === 'exited') return 'exited 0';
  if (run.status === 'stopped') return 'stopped';
  if (run.exitCode !== null && run.exitCode !== 0) return `failed (exit ${run.exitCode})`;
  return run.error ? `failed (${plainLine(run.error)})` : 'failed';
};

const runningLine = (run: RunEntry, last: string): RunLine => ({
  tone: 'success',
  label: last ? RUNNING_LABEL : LAUNCHING_LABEL,
  since: run.startedAt ?? run.queuedAt,
  last,
});

const runLine = (run: RunEntry): RunLine => {
  const last = plainLine(run.lastLine);
  if (run.status === 'queued') return { tone: 'warning', label: QUEUED_LABEL, since: null, last: '' };
  if (run.status === 'running') return runningLine(run, last);
  return { tone: END_TONES[run.status], label: endLabel(run), since: run.endedAt ?? run.queuedAt, last };
};

export { runLine };
