/* @layer renderer-app @kind constants */
const ACCOUNTS_COLL = 'accounts';
const ACCOUNTS_STATE_COLL = 'accountsState';
const DRIFT_DOC = 'drift';

const NONE_FOUND = 'No accounts yet. Argus finds them in the Desktop data folders and the extra account folders.';
const CURRENT_LABEL = 'Current';
const CURRENT_TIP = 'Desktop is signed in here';
const NO_USAGE = 'No usage read yet';

const METER_TONE = { normal: 'primary', warning: 'warning', critical: 'danger' } as const;

const WEEKLY_KIND = 'weekly_all';
const SCOPED_KIND = 'weekly_scoped';
const LIMIT_LABEL: Readonly<Record<string, string>> = {
  weekly_all: 'Weekly, all models',
  five_hour: 'Session (5 h)',
};

const RESET_TIME: Intl.DateTimeFormatOptions = { hour: 'numeric', minute: '2-digit' };
const RESET_DAY: Intl.DateTimeFormatOptions = { weekday: 'short', hour: 'numeric', minute: '2-digit' };
const CLOCK_SECONDS: Intl.DateTimeFormatOptions = { hour: 'numeric', minute: '2-digit', second: '2-digit' };
const LOGIN_DATE: Intl.DateTimeFormatOptions = { month: 'short', day: 'numeric' };
const DAY_MS = 86_400_000;
const CARDS_TICK_MS = 60_000;

const DEFAULT_SWITCH_AT = 98;
const DEFAULT_HANDOVER_PERCENT = 99;
const MIN_PERCENT = 1;
const MAX_PERCENT = 100;

const SWITCH_ACTION = 'Switch Desktop to this account';
const SWITCH_OFF_TEXT = 'Switching here is off.';
const SWITCH_RUNNING_TEXT = 'A switch is running.';
const atLimitText = (clock: string): string => `At limit${clock ? ` until ${clock}` : ''}`;
const movesText = (from: string | null, to: string): string => (from ? `Desktop moves from ${from} to ${to}.` : `Desktop moves to ${to}.`);

const RULES_TITLE = 'Switch rules';
const RULES_SUBTITLE = "Each account's own rules. They apply while Desktop is on that account.";
const AUTO_SWITCH_LABEL = 'Auto-switch';
const AUTO_SWITCH_HINT = 'Leave this account when its use reaches the switch point';
const SWITCH_AT_LABEL = 'Switch at';
const HANDOVER_LABEL = 'Handover';
const HANDOVER_HINT = 'Ask the working leads for a handover before switching';
const ASK_AT_LABEL = 'Ask at';
const CAN_RECEIVE_LABEL = 'Can be switched to';
const CAN_RECEIVE_HINT = 'Other accounts may switch Desktop here';
const HANDOVER_OPTIONS = [
  { value: 'off', label: 'Off' },
  { value: 'percent', label: 'At a percent' },
  { value: 'auto', label: 'Auto' },
];

export {
  ACCOUNTS_COLL, ACCOUNTS_STATE_COLL, ASK_AT_LABEL, atLimitText, AUTO_SWITCH_HINT, AUTO_SWITCH_LABEL, CAN_RECEIVE_HINT,
  CAN_RECEIVE_LABEL, CARDS_TICK_MS, CLOCK_SECONDS, CURRENT_LABEL, CURRENT_TIP, DAY_MS, DEFAULT_HANDOVER_PERCENT, DEFAULT_SWITCH_AT,
  DRIFT_DOC, HANDOVER_HINT, HANDOVER_LABEL, HANDOVER_OPTIONS, LIMIT_LABEL, LOGIN_DATE, MAX_PERCENT, METER_TONE, MIN_PERCENT, movesText, NO_USAGE,
  NONE_FOUND, RESET_DAY, RESET_TIME, RULES_SUBTITLE, RULES_TITLE, SCOPED_KIND, SWITCH_ACTION, SWITCH_AT_LABEL, SWITCH_OFF_TEXT,
  SWITCH_RUNNING_TEXT, WEEKLY_KIND,
};
