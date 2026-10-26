/* @layer renderer-app @kind constants */
import type { ThreadView } from './ThreadDrawer.type';

const WORK_LIMIT = 15;

const FINISHED_STATUSES = ['done', 'dropped'];

const ENDED_THREAD_STATUSES = ['done', 'archived'];

const COORDINATOR_ROLE = 'coordinator';

const NO_THREAD: ThreadView = { thread: null, work: [], counts: { questions: 0, toPickUp: 0 }, paused: false };

export { COORDINATOR_ROLE, ENDED_THREAD_STATUSES, FINISHED_STATUSES, NO_THREAD, WORK_LIMIT };
