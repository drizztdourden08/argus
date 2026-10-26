/* @layer renderer-app @kind constants */
import { defineStatuses } from '@drizztdourden08/tessera';

const NEED_STATUSES: readonly string[] = ['stale', 'merged', 'dirty-stale'];

const DEFAULT_LIMIT_DAYS = 3;

const JOB_WINDOW_MS = 3_600_000;

const SUBJECTS_SHOWN = 3;

const REASONS_SHOWN = 3;

const JOBS_SHOWN = 3;

const UNKNOWN_SIZE = 'unknown size';

const GB_STEP = 102.4;

const UNREACHABLE = 'Dashboard unreachable, retrying.';

const STATUS_TIP: Readonly<Record<string, string>> = {
  active: 'Touched within the stale limit (the newest of the last commit and the index file).',
  stale: 'Not touched for the stale limit or more, and not merged. The thread was told.',
  merged: 'Merged into the default branch and clean, and not touched for the stale limit: it can go. The thread was told.',
  'dirty-stale': 'Not touched for the stale limit or more, with uncommitted changes: never "can go". The thread was told.',
  missing: 'Git still lists it, but its folder is gone (git worktree prune). The thread was told.',
  'not-git': 'A folder under the rule that is not a git work tree.',
  leftover: 'git no longer lists this folder; only files are left',
  deleting: 'Argus is deleting this leftover folder in the background; the row goes when the folder is gone.',
  error: 'Git could not be read in this folder.',
  kept: 'The thread kept it (coord worktree keep): no stale signal until the date.',
};

const WORKTREE_STATUSES = defineStatuses({
  active: { label: 'active', tone: 'success' },
  stale: { label: 'stale', tone: 'warning' },
  merged: { label: 'merged, can go', tone: 'primary' },
  'dirty-stale': { label: 'dirty, stale', tone: 'danger' },
  missing: { label: 'missing', tone: 'warning' },
  'not-git': { label: 'not git', tone: 'neutral' },
  leftover: { label: 'leftover', tone: 'neutral' },
  deleting: { label: 'deleting', tone: 'info', pulse: true },
  error: { label: 'error', tone: 'danger' },
  kept: { label: 'kept', tone: 'info' },
});

const REMOVE_LOOKS = {
  leftover: { label: 'Delete leftover', danger: true, tip: 'Delete this leftover folder now, from here. No backup is possible: git does not know it.' },
  safe: { label: 'Remove', danger: false, tip: 'Remove this worktree folder now, from here: it is clean and its commits stay in the branch, nothing is lost.' },
  discard: { label: 'Delete now', danger: true, tip: 'Delete this worktree now, from here: its uncommitted changes (or the commits on its detached HEAD that no branch keeps) are discarded, after a backup.' },
} as const;

const NOT_REMOVABLE: readonly string[] = ['not-git', 'error', 'leftover', 'deleting'];

export {
  DEFAULT_LIMIT_DAYS, GB_STEP, JOB_WINDOW_MS, JOBS_SHOWN, NEED_STATUSES, NOT_REMOVABLE, REASONS_SHOWN, REMOVE_LOOKS, STATUS_TIP, SUBJECTS_SHOWN,
  UNKNOWN_SIZE, UNREACHABLE, WORKTREE_STATUSES,
};
