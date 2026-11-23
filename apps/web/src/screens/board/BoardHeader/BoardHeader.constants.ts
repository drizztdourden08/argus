/* @layer renderer-app @kind constants */
const STALE_UPDATE_MS = 60 * 60 * 1000;

const LEAD_STATUS_TONES: Readonly<Record<string, 'success' | 'warning'>> = { active: 'success', paused: 'warning' };

const NO_LEAD = 'no lead yet';

const NAME_MAX = 30;

const NAME_EMPTY = 'A board needs a name.';

const NAME_TOO_LONG = `Use at most ${NAME_MAX} characters.`;

const NOTICES_HINT = 'Board notices: what the wake policy kept off the threads (stale items, worktree reviews, faults, notes). They wake nobody; each thread sees a digest with its next prompt.';

const ALERTS_HINT = 'What needs a look on this board: stalled threads, faults, notices and a lost connection.';

const NO_ALERTS = 'Nothing needs attention.';

const WORKTREES_OVER_REASON = 'more are listed than agents running, so some are not cleaned';

const RUNNING_HINT = 'Agents running now. Open the Agents tab.';

export { ALERTS_HINT, LEAD_STATUS_TONES, NAME_EMPTY, NAME_MAX, NAME_TOO_LONG, NO_ALERTS, NO_LEAD, NOTICES_HINT, RUNNING_HINT, STALE_UPDATE_MS, WORKTREES_OVER_REASON };
