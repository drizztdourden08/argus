/* @layer renderer-app @kind constants */
const CHECK_KIND = 'check';

const LOG_TAB = 'log';

const STATUS_TEXT: Readonly<Record<string, string>> = {
  queued: 'queued',
  running: 'running',
  pass: 'passed',
  fail: 'failed',
  cancelled: 'cancelled',
  'skipped-local': 'skipped locally',
};

const RUN_MISSING = 'This check run is not on the board.';

export { CHECK_KIND, LOG_TAB, RUN_MISSING, STATUS_TEXT };
