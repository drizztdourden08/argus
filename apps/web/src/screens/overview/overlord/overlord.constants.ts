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
const STOPPED_CHAT = 'The Overlord is stopped. Start it to send a message.';
const CHAT_PLACEHOLDER = 'Message the Overlord. It reads as Owner (Argus Overlord tab).';
const EMPTY_TRANSCRIPT = 'Nothing has been said yet.';
const SPEND_NOTE = 'An estimate from token counts. The provider console has the real bill.';
const BUDGET_NOTE = 'At the warning level a notification. At 100 percent the Overlord stops and Argus says so.';
const NO_MODE = 'Hosted, API key';

const TALK_WHO: Readonly<Record<'assistant' | 'owner' | 'wake', string>> = { assistant: 'Overlord', owner: 'Owner', wake: 'Wake' };

const CLOCK_MS = 30_000;
const APPROVAL_PARAM = 'approval';
const CHAT_PARAM = 'chat';

export { APPROVAL_PARAM, BUDGET_NOTE, CHAT_PARAM, CHAT_PLACEHOLDER, CLOCK_MS, EMPTY_TRANSCRIPT, NO_MODE, READING, SPEND_NOTE, STATE_STATUSES, STOPPED_CHAT, TALK_WHO };
