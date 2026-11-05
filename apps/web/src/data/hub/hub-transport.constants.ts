/* @layer renderer-app @kind constants */
const HUB_WORKER_NAME = 'argus-hub';
const LEADER_LOCK = 'argus-hub-leader';
const TAB_LOCK_PREFIX = 'argus-tab-';
const BUS_NAME = 'argus-hub';
const INBOX_PREFIX = 'argus-hub:';
const EVERY_TAB = '*';

const BEAT_MS = 10_000;
const DEAD_AFTER_MS = 30_000;
const WORKER_IDLE_MS = 15_000;
const TIER_LIVE_MS = 5_000;

const TAB_TYPES: readonly string[] = ['announce', 'want', 'unwant', 'presence', 'unpresence', 'tail', 'untail', 'bye', 'beat'];
const HUB_TYPES: readonly string[] = ['hub-hello', 'snapshot', 'changes', 'reset', 'status', 'log', 'run', 'tail-end'];

export {
  BEAT_MS,
  BUS_NAME,
  DEAD_AFTER_MS,
  EVERY_TAB,
  HUB_TYPES,
  HUB_WORKER_NAME,
  INBOX_PREFIX,
  LEADER_LOCK,
  TAB_LOCK_PREFIX,
  TAB_TYPES,
  TIER_LIVE_MS,
  WORKER_IDLE_MS,
};
