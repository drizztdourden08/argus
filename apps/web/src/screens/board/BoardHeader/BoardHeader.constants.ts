/* @layer renderer-app @kind constants */
const LEAD_STATUS_TONES: Readonly<Record<string, 'success' | 'warning'>> = { active: 'success', paused: 'warning' };

const NO_LEAD = 'no lead yet';

const NOTICES_LABEL = 'notices';

const NOTICES_HINT = 'Board notices: what the wake policy kept off the threads (stale items, worktree reviews, faults, notes). They wake nobody; each thread sees a digest with its next prompt.';

export { LEAD_STATUS_TONES, NO_LEAD, NOTICES_HINT, NOTICES_LABEL };
