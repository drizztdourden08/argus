/* @layer renderer-app @kind constants */
import { defineStatuses } from '@drizztdourden08/tessera';
import type { OverlordMode } from '../../../data/overlord';

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

const MODE_NOTES: Readonly<Record<OverlordMode, string>> = {
  off: 'The desktop Overlord runs alone.',
  shadow: 'A read-only copy runs on your API key and logs what it would do.',
  live: 'The hosted Overlord takes over and the desktop one ends its turn.',
};
const MODE_CONFIRM = 'This moves the Overlord thread. The desktop Overlord and the hosted one swap places.';

const asMode = (mode: string | null): OverlordMode => (mode === 'shadow' || mode === 'live' ? mode : 'off');
const needsConfirm = (from: OverlordMode, to: OverlordMode) => from === 'live' || to === 'live';

const CLOCK_MS = 30_000;
const APPROVAL_PARAM = 'approval';

export { APPROVAL_PARAM, BUDGET_NOTE, CLOCK_MS, MODE_CONFIRM, MODE_NOTES, NO_MODE, READING, SPEND_NOTE, STATE_STATUSES, asMode, needsConfirm };
