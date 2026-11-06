/* @layer renderer-app @kind constants */
import type { TagLook } from '@drizztdourden08/tessera';

const NEED_STATUSES: readonly string[] = ['stale', 'merged', 'dirty-stale'];

const STALENESS_STATUSES: readonly string[] = ['stale', 'merged', 'dirty-stale', 'kept'];

const DEFAULT_LIMIT_DAYS = 3;

const JOB_WINDOW_MS = 3_600_000;

const SUBJECTS_SHOWN = 3;

const REASONS_SHOWN = 3;

const JOBS_SHOWN = 3;

const UNKNOWN_SIZE = 'unknown size';

const GB_STEP = 102.4;

const UNREACHABLE = 'Dashboard unreachable, retrying.';

const STATUS_TIP: Readonly<Record<string, string>> = {
  stale: 'Not touched for the stale limit or more, and not merged. The thread was told.',
  merged: 'Merged into the default branch and clean, and not touched for the stale limit: it can go. The thread was told.',
  'dirty-stale': 'Not touched for the stale limit or more, with uncommitted changes: never "can go". The thread was told.',
  missing: 'Git still lists it, but its folder is gone (git worktree prune). The thread was told.',
  'not-git': 'A folder under the rule that is not a git work tree.',
  leftover: 'git no longer lists this folder; only files are left',
  deleting: 'Argus is deleting this leftover folder in the background; the row goes when the folder is gone.',
  error: 'Git could not be read in this folder.',
  kept: 'An old board keep: no stale signal until the date. It does not hold the worktree; only the Overlord's exception list does.',
};

const REQUESTS_SHOWN = 8;

const REQUEST_WAITING: readonly string[] = ['queued', 'running'];

const REQUEST_TIP: Readonly<Record<string, string>> = {
  queued: 'Waiting for Argus',
  running: 'Argus is removing them',
  done: 'Every target was handled',
  partial: 'Some were refused by the guards',
  refused: 'The guards refused every target',
};

const SUCCESS_LOOK: TagLook = { variant: 'urgency', color: 'success' };

const WARNING_LOOK: TagLook = { variant: 'urgency', color: 'warning' };

const DANGER_LOOK: TagLook = { variant: 'urgency', color: 'danger' };

const INFO_LOOK: TagLook = { variant: 'urgency', color: 'info' };

const UNKNOWN_STATUS_LOOK: TagLook = { variant: 'normal', color: 'neutral' };

const REQUEST_LOOK: Readonly<Record<string, TagLook>> = {
  partial: DANGER_LOOK,
  refused: DANGER_LOOK,
};

const HYGIENE_LOOK: Readonly<Record<string, TagLook>> = {
  attached: SUCCESS_LOOK,
  removing: INFO_LOOK,
  'awaiting-lead': WARNING_LOOK,
  reported: DANGER_LOOK,
  exception: UNKNOWN_STATUS_LOOK,
};

const REMOVED_SHOWN = 20;

const STATUS_LABEL: Readonly<Record<string, string>> = {
  'dirty-stale': 'dirty, stale',
  'not-git': 'not git',
  merged: 'merged, can go',
};

const REMOVE_LOOKS = {
  leftover: { label: 'Delete leftover', danger: true, tip: 'Delete this leftover folder now, from here. No backup is possible: git does not know it.' },
  safe: { label: 'Remove', danger: false, tip: 'Remove this worktree folder now, from here: it is clean and its commits stay in the branch, nothing is lost.' },
  discard: { label: 'Delete now', danger: true, tip: 'Delete this worktree now, from here: its uncommitted changes (or the commits on its detached HEAD that no branch keeps) are discarded, after a backup.' },
} as const;

const NOT_REMOVABLE: readonly string[] = ['not-git', 'error', 'leftover', 'deleting'];

export {
  DEFAULT_LIMIT_DAYS, GB_STEP, HYGIENE_LOOK, JOB_WINDOW_MS, JOBS_SHOWN, NEED_STATUSES, NOT_REMOVABLE, REASONS_SHOWN, REMOVE_LOOKS, REMOVED_SHOWN, REQUEST_LOOK, REQUEST_TIP, REQUEST_WAITING, REQUESTS_SHOWN, STATUS_TIP, SUBJECTS_SHOWN,
  STALENESS_STATUSES, STATUS_LABEL, UNKNOWN_SIZE, UNKNOWN_STATUS_LOOK, UNREACHABLE,
};
