/* @layer renderer-app @kind constants */
import type { SegmentOption } from '@drizztdourden08/tessera';
import type { BuildLayout } from './builds.type';

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

const MASTER_TITLE = 'Master build';

const MASTER_NOTE = 'The main checkout: mandatory, no link needed.';

const NEEDS_LINK_TITLE = 'Needs a link';

const EMPTY_TEXT = 'Nothing to start yet.';

const NO_MASTER_TIP = 'No master build: the main checkout has no launcher. The thread must set one.';

const MS_PER_SECOND = 1000;

const SECONDS_PER_MINUTE = 60;

const SECONDS_PER_HOUR = 3600;

const LOG_PANE_TAB = 'log';

const PAGE_ORIGIN = 'page';

const LAYOUT_IDS: BuildLayout[] = ['normal', 'compact'];

const LAYOUT_CHOICES: SegmentOption<BuildLayout>[] = [
  { value: 'normal', icon: 'rows-3', hint: { label: 'Normal', description: 'Each build with its details' } },
  { value: 'compact', icon: 'list', hint: { label: 'Compact', description: 'Tighter cards' } },
];

const RUNNING_TITLE = 'Running now';

const NO_MATCH_TEXT = 'No build matches the filters.';

export {
  BUILD_KIND, CLOCK_TICK_MS, DEFAULT_USE_EVERY, DEFAULT_USE_EVERY_MS, EMPTY_TEXT, LAYOUT_CHOICES, LAYOUT_IDS, LOG_PANE_TAB, MASTER, MASTER_NOTE, MASTER_TITLE, MS_PER_SECOND,
  NEEDS_LINK_TITLE, NO_MASTER_TIP, NO_MATCH_TEXT, PAGE_ORIGIN, RETIRED_KEY, RUNNING_TITLE, SECONDS_PER_HOUR, SECONDS_PER_MINUTE, SHELL_NAME, SIGNAL_MAX, STAGE_LABELS, TITLE_WORDS,
};
