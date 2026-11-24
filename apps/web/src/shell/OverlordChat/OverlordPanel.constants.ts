/* @layer renderer-app @kind constants */
import { defineStatuses } from '@drizztdourden08/tessera';
import type { OverlordMode } from '../../data/overlord';

const STATE_STATUSES = defineStatuses({
  stopped: { label: 'Stopped', tone: 'neutral' },
  starting: { label: 'Starting', tone: 'info', pulse: true },
  idle: { label: 'Idle', tone: 'success' },
  turn: { label: 'Working', tone: 'info', pulse: true },
  'paused-budget': { label: 'Paused, budget reached', tone: 'warning' },
  'waiting-approval': { label: 'Waiting for approval', tone: 'warning' },
  crashed: { label: 'Crashed', tone: 'danger' },
});

const NO_LIMIT = 'No limit';
const READING = 'Reading the Overlord.';
const SPEND_NOTE = 'An estimate from token counts. The provider console has the real bill.';
const NO_MODE = 'Hosted, API key';

const MODE_NOTES: Readonly<Record<OverlordMode, string>> = {
  off: 'The desktop Overlord runs alone.',
  shadow: 'A read-only copy runs on your API key and logs what it would do.',
  live: 'The hosted Overlord takes over and the desktop one ends its turn.',
};
const MODE_CONFIRM = 'This moves the Overlord thread. The desktop Overlord and the hosted one swap places.';

const APPROVAL_PARAM = 'approval';

export { APPROVAL_PARAM, MODE_CONFIRM, MODE_NOTES, NO_LIMIT, NO_MODE, READING, SPEND_NOTE, STATE_STATUSES };
