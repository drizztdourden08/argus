/* @layer renderer-app @kind constants */
import { defineStatuses } from '@drizztdourden08/tessera';

const ACCOUNTS_COLL = 'accounts';
const ACCOUNTS_STATE_COLL = 'accountsState';
const DRIFT_DOC = 'drift';

const NONE_FOUND = 'No accounts yet. Argus finds them in the Desktop data folders and the extra account folders.';
const ACTIVE_NOTE = 'Active';
const ACTIVE_TIP = 'Desktop is signed in to this account now';
const NO_DATA = 'No data';

const METER_TONE = { normal: 'primary', warning: 'warning', critical: 'danger' } as const;

const SESSION_KIND = 'five_hour';
const WEEKLY_KIND = 'weekly_all';

const RESET_TIME: Intl.DateTimeFormatOptions = { hour: 'numeric', minute: '2-digit' };
const RESET_DAY: Intl.DateTimeFormatOptions = { weekday: 'short', hour: 'numeric', minute: '2-digit' };
const CLOCK_SECONDS: Intl.DateTimeFormatOptions = { hour: 'numeric', minute: '2-digit', second: '2-digit' };
const DAY_MS = 86_400_000;

const RECEIVE_STATUSES = defineStatuses({
  open: { label: 'Can receive', tone: 'success' },
  locked: { label: 'Locked', tone: 'neutral' },
});

const STRIP_COLUMNS = ['Account', 'Session (5 h)', 'Weekly', 'Receive', ''];

const DRIFT_TIP = 'Differences between the Desktop sidebars of the accounts';

const DEFAULT_SWITCH_AT = 98;
const DEFAULT_HANDOVER_PERCENT = 99;
const MIN_PERCENT = 1;
const MAX_PERCENT = 100;
const AUTO_SWITCH_LABEL = 'Auto-switch';
const HANDOVER_LABEL = 'Handover';
const CAN_RECEIVE_LABEL = 'Can be switched to';
const AT_LIMIT = 'At limit';
const limitUntil = (clock: string): string => `until ${clock}`;
const limitTip = (clock: string): string => `At its weekly limit${clock ? ` until ${clock}` : ''}.`;
const RULES_LABEL = 'Rules';
const SWITCH_LABEL = 'Switch here';
const HANDOVER_OPTIONS = [
  { value: 'off', label: 'Handover off' },
  { value: 'percent', label: 'Handover at a percent' },
  { value: 'auto', label: 'Handover auto' },
];

export {
  AT_LIMIT, limitTip, limitUntil, ACCOUNTS_COLL, ACCOUNTS_STATE_COLL, ACTIVE_NOTE, ACTIVE_TIP, AUTO_SWITCH_LABEL, CAN_RECEIVE_LABEL, CLOCK_SECONDS, DAY_MS,
  DEFAULT_HANDOVER_PERCENT, DEFAULT_SWITCH_AT, DRIFT_DOC, DRIFT_TIP, HANDOVER_LABEL, HANDOVER_OPTIONS, MAX_PERCENT, METER_TONE, MIN_PERCENT,
  NO_DATA, NONE_FOUND, RECEIVE_STATUSES, RESET_DAY, RESET_TIME, RULES_LABEL, SESSION_KIND, STRIP_COLUMNS, SWITCH_LABEL, WEEKLY_KIND,
};
