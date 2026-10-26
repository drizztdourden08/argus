/* @layer renderer-app @kind constants */
import type { ListItemShape, LogKindDef } from '@drizztdourden08/tessera';

const MASTER = 'master';

const DEFAULT_USE_EVERY = '7d';

const DEFAULT_USE_EVERY_MS = 7 * 24 * 60 * 60 * 1000;

const CLOCK_TICK_MS = 60 * 1000;

const RETIRED_KEY = 'coord.lcretired';

const TERMINAL_HEIGHT = 288;

const FULL_LOG_HEIGHT = 520;

const FINISHED_SHOWN = 20;

const LEAD_TEXT = 'What you can start. The registered agent keeps this list true.';

const EMPTY_TEXT = 'Nothing to start yet.';

const EMPTY_HINT = 'The registered agent fills this list: coord launcher set master --cmd "..." for the main checkout, one launcher per build or worktree, and --current on the one to start now. A bundle on the Tests tab links its launcher with coord build set <tag> --launcher <name>.';

const NO_MASTER_TEXT = 'No master launcher: the thread must set one. Every project has a master (the main checkout\'s app): coord launcher set master --cmd "...". Until then no build can be handed to you, and the lead has been told.';

const TERMINAL_KINDS: readonly LogKindDef[] = [
  { id: 'out', label: 'Output' },
  { id: 'err', label: 'Errors', tone: 'danger', toneMessage: true },
  { id: 'sys', label: 'System', tone: 'muted', toneMessage: 'dim' },
];

const RUN_ROW_SHAPE: ListItemShape = { icon: false, columns: 2, action: true };

export {
  CLOCK_TICK_MS, DEFAULT_USE_EVERY, DEFAULT_USE_EVERY_MS, EMPTY_HINT, EMPTY_TEXT, FINISHED_SHOWN, FULL_LOG_HEIGHT, LEAD_TEXT, MASTER, NO_MASTER_TEXT,
  RETIRED_KEY, RUN_ROW_SHAPE, TERMINAL_HEIGHT, TERMINAL_KINDS,
};
