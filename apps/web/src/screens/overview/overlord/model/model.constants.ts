/* @layer renderer-app @kind constants */
import type { OverlordState } from '../../../../data/overlord';
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

const SECONDS_PER_MINUTE = 60;
const MINUTES_PER_HOUR = 60;
const MS_PER_SECOND = 1000;

export { CONTROLS_BY_STATE, MINUTES_PER_HOUR, MS_PER_SECOND, NO_CONTROLS, SECONDS_PER_MINUTE };
