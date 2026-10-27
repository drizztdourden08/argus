/* @layer renderer-app @kind constants */
const MASTER = 'master';

const DEFAULT_USE_EVERY = '7d';

const DEFAULT_USE_EVERY_MS = 7 * 24 * 60 * 60 * 1000;

const CLOCK_TICK_MS = 60 * 1000;

const RETIRED_KEY = 'coord.lcretired';

const SHELL_NAME = typeof navigator !== 'undefined' && /Windows/i.test(navigator.userAgent) ? 'cmd.exe' : 'sh';

const LEAD_TEXT = 'What you can start. The registered agent keeps this list true.';

const EMPTY_TEXT = 'Nothing to start yet.';

const EMPTY_HINT = 'The registered agent fills this list: coord launcher set master --cmd "..." for the main checkout, one launcher per build or worktree, and --current on the one to start now. A bundle on the Tests tab links its launcher with coord build set <tag> --launcher <name>.';

const NO_MASTER_TEXT = 'No master launcher: the thread must set one. Every project has a master (the main checkout\'s app): coord launcher set master --cmd "...". Until then no build can be handed to you, and the lead has been told.';

export {
  CLOCK_TICK_MS, DEFAULT_USE_EVERY, DEFAULT_USE_EVERY_MS, EMPTY_HINT, EMPTY_TEXT, LEAD_TEXT, MASTER, NO_MASTER_TEXT,
  RETIRED_KEY, SHELL_NAME,
};
