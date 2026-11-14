/* @layer renderer-app @kind constants */
import type { OverlordMode, OverlordState } from '../../../../data/overlord';
import type { ControlSet } from './model.type';

const CONTROLS_BY_STATE: Readonly<Record<OverlordState, ControlSet>> = {
  stopped: { start: true, restart: false, stop: false },
  starting: { start: false, restart: false, stop: true },
  idle: { start: false, restart: true, stop: true },
  turn: { start: false, restart: true, stop: true },
  'paused-budget': { start: false, restart: true, stop: true },
  'waiting-approval': { start: false, restart: true, stop: true },
  crashed: { start: true, restart: true, stop: false },
};

const NO_CONTROLS: ControlSet = { start: false, restart: false, stop: false };

const MODES: readonly OverlordMode[] = ['off', 'shadow', 'live'];

const MODE_LABELS: Readonly<Record<OverlordMode, string>> = { off: 'Off', shadow: 'Shadow', live: 'Live' };

const MODE_STEPS: Readonly<Record<OverlordMode, readonly OverlordMode[]>> = {
  off: ['shadow'],
  shadow: ['off', 'live'],
  live: ['off'],
};

const MODE_REFUSALS: Readonly<Record<OverlordMode, string>> = {
  off: '',
  shadow: 'Go to off first',
  live: 'Start shadow first',
};

const SECONDS_PER_MINUTE = 60;
const MINUTES_PER_HOUR = 60;
const MS_PER_SECOND = 1000;

export { CONTROLS_BY_STATE, MINUTES_PER_HOUR, MODES, MODE_LABELS, MODE_REFUSALS, MODE_STEPS, MS_PER_SECOND, NO_CONTROLS, SECONDS_PER_MINUTE };
