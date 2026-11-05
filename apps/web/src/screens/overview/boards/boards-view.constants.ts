/* @layer renderer-app @kind constants */
import type { ListItemShape } from '@drizztdourden08/tessera';
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

const WIDE_QUERY = '(min-width: 1280px)';

const BOARD_COLUMN_MIN = 280;

const ROW_SHAPE: ListItemShape = { icon: true, columns: 3, action: false };

export { ALL_BOARDS, BOARD_COLUMN_MIN, NO_BOARDS, NO_BOARDS_YET, ROW_SHAPE, STATE_LABELS, STATE_ORDER, WIDE_QUERY };
