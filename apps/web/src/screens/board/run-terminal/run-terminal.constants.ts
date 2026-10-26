/* @layer renderer-app @kind constants */
import type { LogKindDef } from '@drizztdourden08/tessera';

const TERMINAL_HEIGHT = 288;

const FULL_LOG_HEIGHT = 520;

const TERMINAL_KINDS: readonly LogKindDef[] = [
  { id: 'out', label: 'Output' },
  { id: 'err', label: 'Errors', tone: 'danger', toneMessage: true },
  { id: 'sys', label: 'System', tone: 'muted', toneMessage: 'dim' },
];

const INPUT_OPEN = 'Type a line for the program and press Enter';

const INPUT_NONE = 'This run has no input';

const INPUT_OVER = 'The run is over';

export { FULL_LOG_HEIGHT, INPUT_NONE, INPUT_OPEN, INPUT_OVER, TERMINAL_HEIGHT, TERMINAL_KINDS };
