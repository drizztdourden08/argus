/* @layer renderer-app @kind constants */
const FAULT_LOOK = { variant: 'urgency', color: 'danger' } as const;

const PLAIN_LOOK = { variant: 'normal', color: 'neutral' } as const;

const STATE_RANK: Readonly<Record<string, number>> = { fault: 0, waking: 1, listening: 2, busy: 2, idle: 3 };

const UNKNOWN_RANK = 4;

const STALE_AGENT_MS = 45 * 60 * 1000;

const NO_LOAD = { working: 0, stalled: 0 };

const BUSY_TIP = 'in a turn right now; its wake listener starts again at its next tool call or when the turn ends';

const DEAF_NOTE = '. The session is active: the Overlord is told when this lasts.';

const CLOSED_BOARD_NOTE = '. Its board tab is not open: the Overlord is told when this lasts.';

const REGISTERED_HINT = 'The threads hooked on this dashboard';

export { BUSY_TIP, CLOSED_BOARD_NOTE, DEAF_NOTE, FAULT_LOOK, NO_LOAD, PLAIN_LOOK, REGISTERED_HINT, STALE_AGENT_MS, STATE_RANK, UNKNOWN_RANK };
