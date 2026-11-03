/* @layer renderer-app @kind constants */
const MESSAGE_MAX = 600;

const REMOVE_PATH: readonly string[] = ['worktrees', 'remove'];

const REQUESTS_PATH: readonly string[] = ['worktrees', 'requests'];

const BATCH_PATH: readonly string[] = ['worktrees', 'leftovers', 'remove'];

export { BATCH_PATH, MESSAGE_MAX, REMOVE_PATH, REQUESTS_PATH };
