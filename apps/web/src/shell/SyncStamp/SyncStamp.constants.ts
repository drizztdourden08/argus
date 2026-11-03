/* @layer renderer-app @kind constants */
const MS_PER_HOUR = 3_600_000;

const SYNC_AMBER_MS = 20 * MS_PER_HOUR;
const SYNC_DUE_MS = 24 * MS_PER_HOUR;

const NEVER_SYNCED = 'No full remote sync recorded yet';

export { NEVER_SYNCED, SYNC_AMBER_MS, SYNC_DUE_MS };
