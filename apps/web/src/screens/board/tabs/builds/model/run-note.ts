/* @layer renderer-app @kind logic */
import { runLine } from '../../../../../data/runs';
import type { RunEntry } from '../../../../../data/runs';

const runNote = (run: RunEntry): string => {
  const last = runLine(run).last;
  return last === '' ? run.error ?? '' : last;
};

export { runNote };
