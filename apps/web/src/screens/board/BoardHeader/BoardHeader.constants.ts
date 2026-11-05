/* @layer renderer-app @kind constants */
const LEAD_STATUS_TONES: Readonly<Record<string, 'success' | 'warning'>> = { active: 'success', paused: 'warning' };

const NO_LEAD = 'no lead yet';

const NOTICES_HINT = 'Board notices: what the wake policy kept off the threads (stale items, worktree reviews, faults, notes). They wake nobody; each thread sees a digest with its next prompt.';

const ALERTS_HINT = 'What needs a look on this board: stalled threads, faults, notices and a lost connection.';

const NO_ALERTS = 'Nothing needs attention.';

const UPDATED_HINT = 'When the board data last changed.';

const WORKTREES_HINT = 'Worktrees listed (main checkouts aside). Open the Worktrees tab.';

const WORKTREES_OVER_HINT = 'More worktrees are listed than agents running: some are not cleaned. Open the Worktrees tab.';

const RUNNING_HINT = 'Agents running now. Open the Agents tab.';

export { ALERTS_HINT, LEAD_STATUS_TONES, NO_ALERTS, NO_LEAD, NOTICES_HINT, RUNNING_HINT, UPDATED_HINT, WORKTREES_HINT, WORKTREES_OVER_HINT };
