/* @layer renderer-app @kind constants */
import type { RunPhase, SyncStepKind } from '../../../data/types';

const SYNC_STEPS_COLL = 'syncSteps';
const RUN_DOC = 'run';
const SYNC_PATH = '/api/accounts/sync';
const UNDO_PATH = '/api/accounts/sync/undo';
const QUIT_CANCEL_PATH = '/api/accounts/switch/cancel';
const QUIT_CANCEL_LABEL = 'Cancel';
const QUIT_FALLBACK = 'Quit Desktop from its tray icon (right-click, Quit). Argus goes on as soon as it has quit.';

const LOG_LIMIT = 3;
const ACTIVE_PHASES: ReadonlySet<RunPhase> = new Set<RunPhase>(['scan', 'waiting_for_quit', 'plan', 'backup', 'apply', 'verify']);

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
  move_chat: 'Move chat',
  delete_chat: 'Delete chat',
  rename_folder: 'Rename folder',
  delete_folder: 'Delete folder',
  set_view: 'Set folder view',
};

const PHASE_LABEL: Readonly<Record<RunPhase, string>> = {
  idle: 'Starting',
  scan: 'Scanning',
  waiting_for_quit: 'Waiting for Desktop to quit',
  plan: 'Planning',
  backup: 'Backing up',
  apply: 'Applying',
  verify: 'Verifying',
  done: 'Done',
  failed: 'Failed',
};

const MATCH_TEXT = 'both accounts match';
const FAILED_TEXT = 'the sync stopped before it finished.';
const NEVER_SYNCED = 'No sync has run yet.';
const SYNC_TITLE = 'Sidebar sync';
const BOTH_MATCH = 'Both sidebars match';
const REVIEW_LABEL = 'Review differences';
const SYNC_LABEL = 'Sync both accounts now';
const COMPARE_PATH = '/api/accounts/compare';
const SAME_AS_CURRENT = 'Same as current';
const DIFF_KINDS: Readonly<Record<string, readonly [string, string]>> = {
  chat: ['chat', 'chats'],
  folder: ['folder', 'folders'],
};
const DIFF_PHRASES: Readonly<Record<string, (what: string) => string>> = {
  missing: (what) => `${what} missing here`,
  only_here: (what) => `${what} only here`,
  active_here_only: (what) => `${what} open here but archived on current`,
  archived_here_only: (what) => `${what} archived here but open on current`,
};
const DRIFT_KINDS: Readonly<Record<string, readonly [string, string]>> = {
  chat: ['chat', 'chats'],
  archive: ['archived', 'archived'],
  folder: ['folder', 'folders'],
  membership: ['folder membership', 'folder memberships'],
  thread: ['sub-thread', 'sub-threads'],
};
const UNDO_LABEL = 'Undo last sync';
const SYNC_STARTED = 'Sidebar sync started.';
const UNDO_STARTED = 'Undoing the last sync.';

export {
  ACTIVE_PHASES, BOTH_MATCH, COMPARE_PATH, DIFF_KINDS, DIFF_PHRASES, DRIFT_KINDS, FAILED_TEXT, LOG_LIMIT,
  MATCH_TEXT, NEVER_SYNCED, PHASE_LABEL, QUIT_CANCEL_LABEL, QUIT_CANCEL_PATH, QUIT_FALLBACK,
  REVIEW_LABEL, RUN_DOC, SAME_AS_CURRENT, STEP_KIND_LABEL, SYNC_LABEL, SYNC_PATH, SYNC_STARTED, SYNC_STEPS_COLL, SYNC_TITLE, UNDO_LABEL, UNDO_PATH, UNDO_STARTED,
};
