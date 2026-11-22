/* @layer renderer-app @kind constants */
import type { OverlordActivity, OverlordState } from '../../data/overlord';

const PRESENCE_LABEL = 'Open the Overlord chat';
const CLOCK_MS = 1_000;
const MS_PER_MINUTE = 60_000;

const ACTIVITY_LABELS: Readonly<Record<OverlordActivity, string>> = {
  off: 'Overlord is off',
  idle: 'Overlord is idle',
  thinking: 'Overlord is thinking',
  working: 'Overlord is working',
};

const ACTIVITY_BY_STATE: Readonly<Record<OverlordState, OverlordActivity>> = {
  stopped: 'off',
  starting: 'off',
  crashed: 'off',
  'paused-budget': 'off',
  idle: 'idle',
  turn: 'thinking',
  'waiting-approval': 'idle',
};

export { ACTIVITY_BY_STATE, ACTIVITY_LABELS, CLOCK_MS, MS_PER_MINUTE, PRESENCE_LABEL };
