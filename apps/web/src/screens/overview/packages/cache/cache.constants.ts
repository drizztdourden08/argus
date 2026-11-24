/* @layer renderer-app @kind constants */
const READING = 'Reading the build cache.';
const NO_FOLDER = 'There is no build cache folder yet. Brock creates it with the first cached build.';
const NONE_CACHED = 'The build cache is empty.';
const NOTHING_TO_PRUNE = 'Nothing is past its grace period, so there is nothing to delete.';
const SHADOW_TEXT = 'Argus runs in shadow mode here: it lists the cache and deletes nothing.';

const CALM_LOOK = { variant: 'normal', color: 'neutral' } as const;
const WARN_LOOK = { variant: 'urgency', color: 'warning' } as const;
const GOOD_LOOK = { variant: 'urgency', color: 'success' } as const;

const USERS_LINES = 2;
const PRUNE_SHOWN = 12;

export { CALM_LOOK, GOOD_LOOK, NONE_CACHED, NOTHING_TO_PRUNE, NO_FOLDER, PRUNE_SHOWN, READING, SHADOW_TEXT, USERS_LINES, WARN_LOOK };
