/* @layer renderer-app @kind constants */
import type { NextRunPhase } from '../../../data/types';

const ACCOUNTS_PATH = '/api/accounts';
const NEXT_PATH = '/api/accounts/next';
const NEXT_TICK_MS = 10_000;
const NEXT_SAVED = 'Next switch saved.';
const NEXT_CLEARED = 'Next switch cleared.';

const TAB_SWITCH = 'switch';
const TAB_SYNC = 'sync';
const ACCOUNT_TABS = [
  { id: TAB_SWITCH, label: 'Switch' },
  { id: TAB_SYNC, label: 'Manual sync' },
];

const DRAG_TYPE = 'application/x-argus-account';
const SLOT_CLEAR = 'Clear';
const SLOT_FROM = 'From';
const SLOT_TO = 'To';
const SLOT_LEFT = 'Left account';
const SLOT_RIGHT = 'Right account';
const STRIP_LABEL = 'Next switch';
const WHEN_UNKNOWN = 'When is not known yet';
const PANE_LABEL = 'Account details';

const STEP_ASKED = 'handover-asked';
const STEP_DONE = 'handover-done';
const STEP_SWITCHED = 'switched';
const STEP_LABELS = { asked: 'Handover asked', done: 'Handover done', limit: 'Limit confirmed', switched: 'Switched' } as const;
const STEP_OF_PHASE: Readonly<Record<NextRunPhase, string | null>> = {
  idle: null,
  'handover-asked': STEP_ASKED,
  'handover-done': STEP_DONE,
  'limit-confirmed': STEP_DONE,
  switching: STEP_SWITCHED,
  switched: STEP_SWITCHED,
  failed: null,
};

const DIRECTION_LTR = 'ltr';
const DIRECTION_RTL = 'rtl';
const DIRECTION_BOTH = 'both';
const DIRECTION_LABEL = 'Direction';
const DIRECTION_OPTIONS = [
  { value: DIRECTION_LTR, label: 'Left to right' },
  { value: DIRECTION_RTL, label: 'Right to left' },
  { value: DIRECTION_BOTH, label: 'Both' },
];
const SYNC_GO_COPY = 'Copy';
const SYNC_GO_REVIEW = 'Review both';

export {
  ACCOUNT_TABS, ACCOUNTS_PATH, DIRECTION_BOTH, DIRECTION_LABEL, DIRECTION_LTR, DIRECTION_OPTIONS, DRAG_TYPE, NEXT_CLEARED, NEXT_PATH,
  NEXT_SAVED, NEXT_TICK_MS, PANE_LABEL, SLOT_CLEAR, SLOT_FROM, SLOT_LEFT, SLOT_RIGHT, SLOT_TO, STEP_ASKED, STEP_DONE, STEP_LABELS,
  STEP_OF_PHASE, STEP_SWITCHED, STRIP_LABEL, SYNC_GO_COPY, SYNC_GO_REVIEW, TAB_SWITCH, TAB_SYNC, WHEN_UNKNOWN,
};
