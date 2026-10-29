/* @layer renderer-app @kind constants */
import { defineStatuses } from '@drizztdourden08/tessera';

const CHECK_STATUSES = defineStatuses({
  queued: { label: 'Queued', tone: 'neutral' },
  running: { label: 'Running', tone: 'info', pulse: true },
  pass: { label: 'Passed', tone: 'success' },
  fail: { label: 'Failed', tone: 'danger' },
  cancelled: { label: 'Cancelled', tone: 'neutral' },
  'skipped-local': { label: 'Skipped locally', tone: 'warning' },
});

const PR_STATUSES = defineStatuses({
  open: { label: 'Open', tone: 'info' },
  merged: { label: 'Merged', tone: 'success' },
  closed: { label: 'Closed', tone: 'neutral' },
});

const LOADING_TEXT = 'Loading the pull requests.';

const EMPTY_TEXT = 'No pull requests yet.';

const EMPTY_HINT = 'A pull request is a record on this board: the registered agent opens one with coord pr open, and it merges after a passing check run.';

const CHECKS_READING = 'Reading the check runs.';

const CHECKS_NONE = 'No check run yet.';

const LOG_READING = 'Reading the log.';

const LOG_EMPTY = 'This step wrote no log.';

export { CHECK_STATUSES, CHECKS_NONE, CHECKS_READING, EMPTY_HINT, EMPTY_TEXT, LOADING_TEXT, LOG_EMPTY, LOG_READING, PR_STATUSES };
