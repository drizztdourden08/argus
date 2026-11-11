/* @layer renderer-app @kind constants */
import type { SettingsAccount } from '../../../../data/types';

const ACCOUNTS_COLL = 'accounts';
const ACCOUNTS_TITLE = 'Desktop accounts';
const ACCOUNTS_NOTE = 'Switching Desktop between accounts. A new account is discovered fully on the next Argus start.';
const ADD_LABEL = 'Add account';
const SAVE_LABEL = 'Save';
const RESET_K_LABEL = 'Reset k';
const SAVED_TEXT = 'Accounts settings saved.';
const NEW_KEY_HINT = 'Short key';
const DEFAULT_COLOR = '#8a8f98';

const HANDOVER_CHOICES = [
  { value: 'off', label: 'Handover off' },
  { value: 'percent', label: 'Handover at a percent' },
  { value: 'auto', label: 'Handover auto' },
];

const ARCHIVE_CHOICES = [
  { value: 'newest', label: 'The newest record decides' },
  { value: 'anywhere', label: 'Archived anywhere means archived everywhere' },
];

const THRESHOLD_FIELDS = [
  { id: 'warnAt', label: 'Warn at (%)' },
  { id: 'proposeAt', label: 'Propose a switch at (%)' },
  { id: 'fastAt', label: 'Poll fast from (%)' },
  { id: 'pollMinutes', label: 'Poll every (minutes)' },
  { id: 'fastPollSeconds', label: 'Fast poll every (seconds)' },
] as const;

const ESTIMATE_FIELDS = [
  { id: 'opus', label: 'Opus agent' },
  { id: 'sonnet', label: 'Sonnet agent' },
  { id: 'haiku', label: 'Haiku agent' },
  { id: 'lead', label: 'Lead thread' },
  { id: 'safety', label: 'Safety margin' },
] as const;

const BLANK_ACCOUNT: Omit<SettingsAccount, 'key' | 'label' | 'cliDir'> = {
  color: DEFAULT_COLOR,
  canReceive: true,
  rules: { autoSwitch: false, switchAt: 98, handover: 'off' },
};

export {
  ACCOUNTS_COLL, ACCOUNTS_NOTE, ACCOUNTS_TITLE, ADD_LABEL, ARCHIVE_CHOICES, BLANK_ACCOUNT, DEFAULT_COLOR, ESTIMATE_FIELDS, HANDOVER_CHOICES,
  NEW_KEY_HINT, RESET_K_LABEL, SAVED_TEXT, SAVE_LABEL, THRESHOLD_FIELDS,
};
