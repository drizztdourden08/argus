/* @layer renderer-app @kind constants */
import type { LeadState } from './model';

const STATE_ORDER: readonly LeadState[] = ['running', 'idle', 'stale', 'paused', 'not started'];

const STATE_LABELS: Readonly<Record<LeadState, string>> = {
  running: 'Running',
  idle: 'Idle',
  stale: 'Stale',
  paused: 'Paused',
  'not started': 'Not started',
};

const ALL_BOARDS = 'All boards';

const NO_BOARDS = 'No board matches.';

const NO_BOARDS_YET = 'No project has a board yet.';

const RAIL_BREAKPOINT = 1200;
const RAIL_LABEL = 'Now';

const BOARD_COLUMN_MIN = 280;

export { ALL_BOARDS, BOARD_COLUMN_MIN, NO_BOARDS, NO_BOARDS_YET, RAIL_BREAKPOINT, RAIL_LABEL, STATE_LABELS, STATE_ORDER };
