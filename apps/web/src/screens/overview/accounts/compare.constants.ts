/* @layer renderer-app @kind constants */
import type { CompareStatus, RunPhase, SyncStepKind } from '../../../data/types';
import type { ChipLook } from './compare.type';

const SYNC_STEPS_COLL = 'syncSteps';
const RUN_DOC = 'run';
const COMPARE_PATH = '/api/accounts/compare';
const SYNC_PATH = '/api/accounts/sync';
const UNDO_PATH = '/api/accounts/sync/undo';

const LOG_LIMIT = 30;
const INDENT = '\u2003';
const ACTIVE_PHASES: ReadonlySet<RunPhase> = new Set<RunPhase>(['scan', 'plan', 'backup', 'apply', 'verify']);

const STATUS_LOOK: Readonly<Record<CompareStatus, ChipLook>> = {
  same: { label: 'same', tone: 'success' },
  subthread: { label: '', tone: 'neutral' },
  missing: { label: 'will be created', tone: 'info' },
  will_create: { label: 'will be created', tone: 'info' },
  only_here: { label: 'only here', tone: 'warning' },
  not_subthread: { label: 'not a sub-thread here', tone: 'warning' },
  archived_here_only: { label: 'archived here only', tone: 'warning' },
  active_here_only: { label: 'active here', tone: 'warning' },
  folder_differs: { label: 'different folder', tone: 'warning' },
};

const GHOST_STATUSES: ReadonlySet<CompareStatus> = new Set<CompareStatus>(['missing', 'will_create']);
const QUIET_STATUSES: ReadonlySet<CompareStatus> = new Set<CompareStatus>(['same', 'subthread']);

const STEP_KIND_LABEL: Readonly<Record<SyncStepKind, string>> = {
  backup: 'Backup',
  copy_session: 'Copy chat',
  update_session: 'Update chat',
  link_thread: 'Link sub-thread',
  archive: 'Set archived',
  create_folder: 'Create folder',
  assign_folder: 'File in folder',
  write_storage: 'Write folder storage',
  verify: 'Verify',
};

const STEP_KINDS = Object.keys(STEP_KIND_LABEL) as SyncStepKind[];

const PHASE_LABEL: Readonly<Record<RunPhase, string>> = {
  idle: 'Starting',
  scan: 'Scanning',
  plan: 'Planning',
  backup: 'Backing up',
  apply: 'Applying',
  verify: 'Verifying',
  done: 'Done',
  failed: 'Failed',
};

const MATCH_TEXT = 'Both accounts match';
const FAILED_TEXT = 'The sync stopped before it finished.';
const NOTHING_TEXT = 'Nothing to change.';
const EMPTY_COMPARE = 'No chats or folders found in the accounts.';

export {
  ACTIVE_PHASES, COMPARE_PATH, EMPTY_COMPARE, FAILED_TEXT, GHOST_STATUSES, INDENT, LOG_LIMIT, MATCH_TEXT, NOTHING_TEXT, PHASE_LABEL,
  QUIET_STATUSES, RUN_DOC, STATUS_LOOK, STEP_KIND_LABEL, STEP_KINDS, SYNC_PATH, SYNC_STEPS_COLL, UNDO_PATH,
};
