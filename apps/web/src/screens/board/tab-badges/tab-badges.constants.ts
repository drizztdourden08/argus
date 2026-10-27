/* @layer renderer-app @kind constants */
const ACTIVE_AGENT_STATUSES: readonly string[] = ['working', 'waiting', 'stop-requested'];

const NEEDS_DECISION_STATUSES: readonly string[] = ['stale', 'merged', 'dirty-stale', 'missing'];

const STALE_AGENT_MS = 45 * 60 * 1000;

const CLOCK_MS = 60_000;

export { ACTIVE_AGENT_STATUSES, CLOCK_MS, NEEDS_DECISION_STATUSES, STALE_AGENT_MS };
