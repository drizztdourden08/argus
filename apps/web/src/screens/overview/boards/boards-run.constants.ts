/* @layer renderer-app @kind constants */

const WARN_LOOK = { variant: 'urgency', color: 'warning' } as const;
const GOOD_LOOK = { variant: 'urgency', color: 'success' } as const;
const INFO_LOOK = { variant: 'urgency', color: 'info' } as const;

const NO_PLAY = 'master has no Play: its command spans several lines, or its run name is taken (coord launcher show master)';

const RUNS_REFRESH_MS = 250;

export { GOOD_LOOK, INFO_LOOK, NO_PLAY, RUNS_REFRESH_MS, WARN_LOOK };
