/* @layer renderer-app @kind constants */
const MASTER = 'master';

const BUILD_KIND = 'build';

const TITLE_WORDS = 6;

const SIGNAL_MAX = 4;

const DEFAULT_USE_EVERY = '7d';

const DEFAULT_USE_EVERY_MS = 7 * 24 * 60 * 60 * 1000;

const CLOCK_TICK_MS = 60 * 1000;

const RETIRED_KEY = 'coord.lcretired';

const SHELL_NAME = typeof navigator !== 'undefined' && /Windows/i.test(navigator.userAgent) ? 'cmd.exe' : 'sh';

const STAGE_LABELS: Readonly<Record<string, string>> = { draft: 'Draft', ready: 'Ready', closed: 'Closed' };

const LEAD_TEXT = 'What you can start, grouped by the item each build serves. A build closes when its item closes.';

const MASTER_TITLE = 'Master build';

const MASTER_NOTE = 'The main checkout: mandatory, no link needed.';

const NEEDS_LINK_TITLE = 'Needs a link';

const EMPTY_TEXT = 'Nothing to start yet.';

const EMPTY_HINT = 'The registered agent fills this list: coord launcher set master --cmd "..." for the main checkout, and one build per item with --for I-n. --current marks the one to start now.';

const NO_MASTER_TEXT = 'No master launcher: the thread must set one. Every project has a master (the main checkout\'s app): coord launcher set master --cmd "...". Until then no build can be handed to you, and the lead has been told.';

export {
  BUILD_KIND, CLOCK_TICK_MS, DEFAULT_USE_EVERY, DEFAULT_USE_EVERY_MS, EMPTY_HINT, EMPTY_TEXT, LEAD_TEXT, MASTER, MASTER_NOTE, MASTER_TITLE,
  NEEDS_LINK_TITLE, NO_MASTER_TEXT, RETIRED_KEY, SHELL_NAME, SIGNAL_MAX, STAGE_LABELS, TITLE_WORDS,
};
