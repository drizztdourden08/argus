/* @layer renderer-app @kind constants */
import { defineStatuses } from '@drizztdourden08/tessera';

const STATE_STATUSES = defineStatuses({
  stopped: { label: 'Stopped', tone: 'neutral' },
  starting: { label: 'Starting', tone: 'info', pulse: true },
  idle: { label: 'Idle', tone: 'success' },
  turn: { label: 'Working', tone: 'info', pulse: true },
  'paused-budget': { label: 'Paused, budget reached', tone: 'warning' },
  'waiting-approval': { label: 'Waiting for approval', tone: 'warning' },
  crashed: { label: 'Crashed', tone: 'danger' },
});

const READING = 'Reading the Overlord.';
const SPEND_NOTE = 'An estimate from token counts. The provider console has the real bill.';
const BUDGET_NOTE = 'At the warning level a notification. At 100 percent the Overlord stops and Argus says so.';
const NO_MODE = 'Hosted, API key';

const CLOCK_MS = 30_000;
const APPROVAL_PARAM = 'approval';

export { APPROVAL_PARAM, BUDGET_NOTE, CLOCK_MS, NO_MODE, READING, SPEND_NOTE, STATE_STATUSES };
