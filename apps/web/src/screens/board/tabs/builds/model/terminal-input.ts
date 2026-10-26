/* @layer renderer-app @kind logic */
import type { RunEntry } from '../../../../../data/runs';
import { INPUT_NONE, INPUT_OPEN, INPUT_OVER } from '../builds.constants';
import type { TerminalInput } from './model.type';

const terminalInput = (run: RunEntry): TerminalInput => {
  if (run.status === 'running' && run.stdin === true) return { enabled: true, placeholder: INPUT_OPEN };
  return { enabled: false, placeholder: run.status === 'running' ? INPUT_NONE : INPUT_OVER };
};

export { terminalInput };
