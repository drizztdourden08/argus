/* @layer renderer-app @kind logic */
import { isLiveRun, runLine } from '../../../../data/runs';
import type { RunEntry } from '../../../../data/runs';

const terminalLast = (run: RunEntry): string => {
  const line = runLine(run);
  if (line.last) return line.last;
  return isLiveRun(run) ? line.label : '';
};

export { terminalLast };
