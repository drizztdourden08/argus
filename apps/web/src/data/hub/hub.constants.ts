/* @layer renderer-app @kind constants */
const MUX_URL = '/api/mux';
const HEALTH_URL = '/api/health';
const MUX_VERSION = 1;
const MAX_BATCH = 50;
const MUX_EVENTS = ['hello', 'snapshot', 'changes', 'ready', 'reset', 'log', 'run', 'tail-end'] as const;

const FAILURES_BEFORE_UNAVAILABLE = 3;
const RECONNECT_BASE_MS = 500;
const RECONNECT_CAP_MS = 8000;
const GONE_RETRIES_AT_ONCE = 1;

export {
  FAILURES_BEFORE_UNAVAILABLE,
  GONE_RETRIES_AT_ONCE,
  HEALTH_URL,
  MAX_BATCH,
  MUX_EVENTS,
  MUX_URL,
  MUX_VERSION,
  RECONNECT_BASE_MS,
  RECONNECT_CAP_MS,
};
