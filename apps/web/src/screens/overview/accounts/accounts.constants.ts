/* @layer renderer-app @kind constants */
import type { AccountSeverity } from '../../../data/types';

const ACCOUNTS_COLL = 'accounts';
const ACCOUNTS_STATE_COLL = 'accountsState';
const DRIFT_DOC = 'drift';

const NONE_FOUND = 'No accounts yet. Argus finds them in the Desktop data folders and the extra account folders.';
const ACTIVE_NOTE = 'Desktop is on this one';
const BINDING_NOTE = 'Binding';
const BINDING_TIP = 'The limit that stops this account now';

const SEVERITY_LOOK: Readonly<Record<AccountSeverity, { color: 'success' | 'warning' | 'danger'; label: string }>> = {
  normal: { color: 'success', label: 'Has room' },
  warning: { color: 'warning', label: 'Getting full' },
  critical: { color: 'danger', label: 'At a limit' },
};

const METER_TONE = { normal: 'primary', warning: 'warning', critical: 'danger' } as const;

const KIND_LABEL: Readonly<Record<string, string>> = {
  five_hour: 'Session (5 h)',
  weekly_all: 'Weekly',
  weekly_scoped: 'Weekly',
};

const RESET_FORMAT: Intl.DateTimeFormatOptions = { weekday: 'short', month: 'short', day: 'numeric', hour: 'numeric', minute: '2-digit' };

const DRIFT_TIP = 'Differences between the Desktop sidebars of the accounts';

const DEFAULT_SWITCH_AT = 98;
const DEFAULT_HANDOVER_PERCENT = 99;
const MIN_PERCENT = 1;
const MAX_PERCENT = 100;
const AUTO_SWITCH_LABEL = 'Auto-switch';
const HANDOVER_LABEL = 'Handover';
const CAN_RECEIVE_LABEL = 'Can be switched to';
const HANDOVER_OPTIONS = [
  { value: 'off', label: 'Handover off' },
  { value: 'percent', label: 'Handover at a percent' },
  { value: 'auto', label: 'Handover auto' },
];

export {
  AUTO_SWITCH_LABEL, CAN_RECEIVE_LABEL, DEFAULT_HANDOVER_PERCENT, DEFAULT_SWITCH_AT, HANDOVER_LABEL, HANDOVER_OPTIONS, MAX_PERCENT, MIN_PERCENT,
  ACCOUNTS_COLL, ACCOUNTS_STATE_COLL, ACTIVE_NOTE, BINDING_NOTE, BINDING_TIP, DRIFT_DOC, DRIFT_TIP, KIND_LABEL, METER_TONE, NONE_FOUND, RESET_FORMAT, SEVERITY_LOOK,
};
