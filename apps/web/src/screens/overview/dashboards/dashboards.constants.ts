/* @layer renderer-app @kind constants */
const SENT_LIMIT = 40;

const WARN_LOOK = { variant: 'urgency', color: 'warning' } as const;
const CALM_LOOK = { variant: 'normal', color: 'neutral' } as const;
const GOOD_LOOK = { variant: 'urgency', color: 'success' } as const;
const BAD_LOOK = { variant: 'urgency', color: 'danger' } as const;
const INFO_LOOK = { variant: 'urgency', color: 'info' } as const;

const STATE_LOOK = {
  running: GOOD_LOOK,
  idle: CALM_LOOK,
  paused: INFO_LOOK,
  stale: WARN_LOOK,
  'not started': CALM_LOOK,
} as const;

const NO_PROJECT = 'No project has a board yet.';
const NOTHING_SENT = 'Nothing sent down yet.';

const NO_PLAY = 'master has no Play: its command spans several lines, or its run name is taken (coord launcher show master)';

const KIND_WORD: Readonly<Record<string, string>> = { decision: 'decision', followup: 'note' };

export {
  BAD_LOOK, CALM_LOOK, GOOD_LOOK, INFO_LOOK, KIND_WORD, NO_PLAY, NO_PROJECT, NOTHING_SENT, SENT_LIMIT, STATE_LOOK, WARN_LOOK,
};
