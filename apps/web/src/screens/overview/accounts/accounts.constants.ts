/* @layer renderer-app @kind constants */
import type { AccountSeverity } from '../../../data/types';

const ACCOUNTS_COLL = 'accounts';
const ACCOUNTS_STATE_COLL = 'accountsState';
const DRIFT_DOC = 'drift';

const NONE_FOUND = 'No Claude accounts yet. Argus finds them under Claude-Data and .claude-accounts.';
const ACTIVE_NOTE = 'Desktop is on this one';

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

const DRIFT_TIP = 'Differences between the Claude Desktop sidebars of the accounts';

export {
  ACCOUNTS_COLL, ACCOUNTS_STATE_COLL, ACTIVE_NOTE, DRIFT_DOC, DRIFT_TIP, KIND_LABEL, METER_TONE, NONE_FOUND, RESET_FORMAT, SEVERITY_LOOK,
};
