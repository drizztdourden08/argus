/* @layer renderer-app @kind constants */
import { defineStatuses } from '@drizztdourden08/tessera';
import type { SwitchPhase, SwitchStage, SwitchSyncCounts } from '../../../../data/types';

const SWITCH_DOC = 'switch';
const SWITCH_PATH = '/api/accounts/switch';
const ANYWAY_PATH = '/api/accounts/switch/anyway';
const CANCEL_PATH = '/api/accounts/switch/cancel';

const SWITCH_LINK_PARAM = 'switch';
const TAB_LINK_PARAM = 'tab';

const RUNNING_PHASES: ReadonlySet<SwitchPhase> = new Set<SwitchPhase>(['countdown', 'handover', 'sync', 'switching', 'verifying']);
const CANCELABLE_PHASES: ReadonlySet<SwitchPhase> = new Set<SwitchPhase>(['countdown', 'handover']);

const STAGES: readonly SwitchStage[] = ['detect', 'countdown', 'handover', 'sync', 'switch', 'verify'];

const PHASE_STAGE: Readonly<Partial<Record<SwitchPhase, SwitchStage>>> = {
  countdown: 'countdown',
  handover: 'handover',
  sync: 'sync',
  switching: 'switch',
  verifying: 'verify',
};

const STAGE_LABEL: Readonly<Record<SwitchStage, string>> = {
  detect: 'Detect',
  countdown: 'Countdown',
  handover: 'Handover',
  sync: 'Sync',
  switch: 'Switch',
  verify: 'Verify',
};

const STAGE_PLAN: Readonly<Record<SwitchStage, string>> = {
  detect: 'Find the threads that are working now.',
  countdown: 'Wait a moment, so the switch can still be cancelled.',
  handover: 'Ask each working thread to write a handover.',
  sync: 'Copy the Desktop sidebar to the new account.',
  switch: 'Quit Desktop and sign in to the new account.',
  verify: 'Check that Desktop is on the new account.',
};

const STEP_STATUSES = defineStatuses({
  waiting: { label: 'Waiting', tone: 'neutral' },
  running: { label: 'Running', tone: 'info', pulse: true },
  done: { label: 'Done', tone: 'success' },
  failed: { label: 'Failed', tone: 'danger' },
  skipped: { label: 'Skipped', tone: 'neutral' },
});

const THREAD_STATUSES = defineStatuses({
  found: { label: 'Found', tone: 'neutral' },
  asked: { label: 'Asked', tone: 'info', pulse: true },
  ready: { label: 'Ready', tone: 'success' },
  skipped: { label: 'Skipped', tone: 'neutral' },
  failed: { label: 'Failed', tone: 'danger' },
});

const SYNC_STATUSES = defineStatuses({
  pending: { label: 'Pending', tone: 'neutral' },
  running: { label: 'Running', tone: 'info', pulse: true },
  done: { label: 'Done', tone: 'success' },
  failed: { label: 'Failed', tone: 'danger' },
  skipped: { label: 'Skipped', tone: 'neutral' },
});

const PHASE_STATUSES = defineStatuses({
  idle: { label: 'Idle', tone: 'neutral' },
  countdown: { label: 'Counting down', tone: 'warning', pulse: true },
  handover: { label: 'Handover', tone: 'info', pulse: true },
  sync: { label: 'Syncing', tone: 'info', pulse: true },
  switching: { label: 'Switching', tone: 'info', pulse: true },
  verifying: { label: 'Verifying', tone: 'info', pulse: true },
  done: { label: 'Done', tone: 'success' },
  failed: { label: 'Failed', tone: 'danger' },
  cancelled: { label: 'Cancelled', tone: 'neutral' },
});

const COUNT_KEYS: readonly (keyof SwitchSyncCounts)[] = ['copied', 'updated', 'folders', 'archived'];

const PROCESS_TITLE = 'Switch process';
const NO_SWITCH = 'No switch running.';
const NO_RUN_YET = 'No switch has run yet.';
const LAST_RUN = 'Last run';
const NOT_REACHED = 'Not reached.';
const ZERO_GRACE_MS = 3000;
const ZERO_TEXT = 'The countdown is at 0:00 and nothing moved.';
const NO_REASON = 'Argus gave no reason.';
const AT_ZERO_HANDOVER = 'At 0:00 Argus asks the working threads for a handover.';
const AT_ZERO_SYNC = 'At 0:00 Argus syncs the sidebar, then switches.';
const THREADS_TITLE = 'Threads for handover';
const NO_THREADS = 'No working threads found.';
const THREADS_UNCHECKED = 'Not checked yet.';
const SYNC_TITLE = 'Sidebar sync';
const SYNC_UNSTARTED = 'Not started yet.';
const STUCK_PREFIX = 'Stuck:';
const BY_RULE = 'Started by the auto-switch rule.';
const BY_OWNER = 'Started by you.';

const HANDOVER_TOGGLE = 'Ask working threads for a handover first';
const START_LABEL = 'Start switch';
const CANCEL_LABEL = 'Cancel';
const ANYWAY_LABEL = 'Switch anyway';
const NOW_LABEL = 'Switch now';
const SWITCH_STARTED = 'Switch started.';
const ANYWAY_TOAST = 'Not waiting for the rest.';
const SWITCH_CANCELLED_TOAST = 'Switch cancelled.';

export {
  ANYWAY_LABEL, ANYWAY_PATH, ANYWAY_TOAST, AT_ZERO_HANDOVER, AT_ZERO_SYNC, BY_OWNER, BY_RULE, CANCELABLE_PHASES, CANCEL_LABEL, CANCEL_PATH, COUNT_KEYS,
  HANDOVER_TOGGLE, LAST_RUN, NO_REASON, NO_RUN_YET, NO_SWITCH, NO_THREADS, NOT_REACHED, NOW_LABEL, PHASE_STAGE, PHASE_STATUSES, PROCESS_TITLE, RUNNING_PHASES,
  STAGE_LABEL, STAGE_PLAN, STAGES, START_LABEL, STEP_STATUSES, STUCK_PREFIX, SWITCH_CANCELLED_TOAST, SWITCH_DOC, SWITCH_LINK_PARAM, SWITCH_PATH,
  SWITCH_STARTED, SYNC_STATUSES, SYNC_TITLE, SYNC_UNSTARTED, TAB_LINK_PARAM, THREAD_STATUSES, THREADS_TITLE, THREADS_UNCHECKED, ZERO_GRACE_MS, ZERO_TEXT,
};
