/* @layer renderer-app @kind logic */
import { runLine } from '../../../../data/runs';
import type { RunEntry } from '../../../../data/runs';

const stateWord = (run: RunEntry): string => {
  if (run.status === 'running') return 'running';
  return run.status === 'queued' ? 'starting' : runLine(run).label;
};

const terminalMeta = (run: RunEntry): string => {
  const input = run.status === 'running' && run.stdin ? ' · input on' : '';
  return `${run.id} · ${stateWord(run)}${input}`;
};

export { terminalMeta };
