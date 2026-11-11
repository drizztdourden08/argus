/* @layer renderer-app @kind constants */
import type { HandoverState, SwitchPhase, SwitchStepStatus } from '../../../../data/types';

const SWITCH_DOC = 'switch';
const CYCLE_DOC = 'cycle';
const HANDOVERS_COLL = 'handovers';
const SWITCH_PATH = '/api/accounts/switch';
const ANYWAY_PATH = '/api/accounts/switch/anyway';
const CANCEL_PATH = '/api/accounts/switch/cancel';

const HANDOVER_STEP = 'handover';
const SWITCH_LINK_PARAM = 'switch';
const TAB_LINK_PARAM = 'tab';

const FINISHED_PHASES: ReadonlySet<SwitchPhase> = new Set<SwitchPhase>(['idle', 'done', 'cancelled', 'failed']);
const CANCELABLE_PHASES: ReadonlySet<SwitchPhase> = new Set<SwitchPhase>(['countdown', 'handover']);
const READY_STATES: ReadonlySet<HandoverState> = new Set<HandoverState>(['ready', 'cut_off', 'resumed', 'not_resumed']);

const STEP_MARK: Readonly<Record<SwitchStepStatus, string>> = {
  pending: '\u25CB',
  running: '\u25CF',
  done: '\u2713',
  failed: '\u2717',
  skipped: '\u2212',
};

const STEP_TONE: Readonly<Record<SwitchStepStatus, 'muted' | 'info' | 'success' | 'danger'>> = {
  pending: 'muted',
  running: 'info',
  done: 'success',
  failed: 'danger',
  skipped: 'muted',
};

const STATE_LABEL: Readonly<Record<HandoverState, string>> = {
  estimate_asked: 'estimate asked',
  estimated: 'estimated',
  asked: 'asked',
  writing: 'writing',
  ready: 'ready',
  cut_off: 'cut off',
  resumed: 'resumed',
  not_resumed: 'not resumed',
};

const STATE_TONE: Readonly<Record<HandoverState, 'info' | 'success' | 'warning'>> = {
  estimate_asked: 'info',
  estimated: 'info',
  asked: 'info',
  writing: 'warning',
  ready: 'success',
  cut_off: 'warning',
  resumed: 'success',
  not_resumed: 'warning',
};

const DIALOG_TITLE = 'Switch Desktop to';
const HANDOVER_TOGGLE = 'Ask working leads for a handover first';
const HANDOVER_TOGGLE_HELP = 'Each lead writes down where it is, and the new account picks the work up from those notes.';
const START_LABEL = 'Start';
const CANCEL_LABEL = 'Cancel';
const CLOSE_LABEL = 'Close';
const ANYWAY_LABEL = 'Switch anyway';
const NOW_LABEL = 'Switch now';
const DESKTOP_CLOSES = 'The Desktop app closes in the next step. This page stays up and keeps following the switch.';
const SWITCH_DONE = 'Desktop is on the new account.';
const SWITCH_CANCELLED = 'The switch was cancelled.';
const ANYWAY_TOAST = 'Not waiting for the rest.';
const CYCLE_TITLE = 'Handover cycle';
const NOT_ASKED_TEXT = 'not asked';
const SWITCH_FAILED = 'The switch failed.';
const REASON_RULE = 'The auto-switch rule reached its percent. Handovers, if asked, are in.';
const REASON_OWNER = 'You asked for this switch.';
const SWITCH_STARTED = 'Switch started.';
const SWITCH_CANCELLED_TOAST = 'Switch cancelled.';


export {
  ANYWAY_LABEL, ANYWAY_PATH, ANYWAY_TOAST, CANCELABLE_PHASES, CANCEL_LABEL, CANCEL_PATH, CLOSE_LABEL, CYCLE_DOC, CYCLE_TITLE, DESKTOP_CLOSES, DIALOG_TITLE,
  FINISHED_PHASES, HANDOVERS_COLL, HANDOVER_STEP, HANDOVER_TOGGLE, HANDOVER_TOGGLE_HELP, NOT_ASKED_TEXT, NOW_LABEL, READY_STATES, REASON_OWNER, REASON_RULE, START_LABEL, STATE_LABEL,
  STATE_TONE, STEP_MARK, STEP_TONE, SWITCH_CANCELLED, SWITCH_CANCELLED_TOAST, SWITCH_DOC, SWITCH_DONE, SWITCH_LINK_PARAM, SWITCH_PATH,
  SWITCH_FAILED, SWITCH_STARTED, TAB_LINK_PARAM,
};
