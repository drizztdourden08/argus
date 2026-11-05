/* @layer renderer-app @kind constants */
import type { BadgeColor } from '@drizztdourden08/tessera';
import type { RunTone } from '../../../data/runs';

const WARN_LOOK = { variant: 'urgency', color: 'warning' } as const;
const GOOD_LOOK = { variant: 'urgency', color: 'success' } as const;
const INFO_LOOK = { variant: 'urgency', color: 'info' } as const;

const RUN_DOT: Readonly<Record<RunTone, BadgeColor>> = {
  success: 'success',
  info: 'info',
  warning: 'warning',
  danger: 'danger',
  neutral: 'tame',
};

const NO_PLAY = 'master has no Play: its command spans several lines, or its run name is taken (coord launcher show master)';

const RUNS_REFRESH_MS = 250;

export { GOOD_LOOK, INFO_LOOK, NO_PLAY, RUN_DOT, RUNS_REFRESH_MS, WARN_LOOK };
