/* @layer renderer-app @kind constants */
import type { BadgeColor, IconName } from '@drizztdourden08/tessera';
import type { LeadState, SignalId, SignalTone } from './model';
import type { StripTone } from './boards.type';

const SIGNAL_ICON: Readonly<Record<SignalId, IconName>> = {
  'needs-you': 'bell',
  running: 'play',
  failing: 'circle-x',
  blocked: 'circle-alert',
  stalled: 'clock',
  fault: 'triangle-alert',
  'sync-due': 'refresh-cw',
  'master-missing': 'wrench',
  paused: 'pause',
  archived: 'archive',
};

const SIGNAL_TONE: Readonly<Record<SignalTone, StripTone>> = {
  bad: 'danger',
  warn: 'warning',
  good: 'success',
  neutral: 'neutral',
  muted: 'neutral',
};

const LEAD_DOT: Readonly<Record<LeadState, BadgeColor>> = {
  running: 'success',
  idle: 'warning',
  stale: 'warning',
  paused: 'warning',
  'not started': 'tame',
};

const FAULT_DOT: BadgeColor = 'danger';

const SIGNAL_MAX = 4;

const NEEDS_BOARD: readonly string[] = ['open', 'send', 'play', 'stop'];

export { FAULT_DOT, LEAD_DOT, NEEDS_BOARD, SIGNAL_ICON, SIGNAL_MAX, SIGNAL_TONE };
