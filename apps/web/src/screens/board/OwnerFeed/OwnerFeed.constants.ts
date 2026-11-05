/* @layer renderer-app @kind constants */
import type { FeedFilter } from './OwnerFeed.type';

const FILTER_STORAGE_KEY = 'coord.feed';

const FILTER_OPTIONS: { value: FeedFilter; label: string }[] = [
  { value: 'all', label: 'All' },
  { value: 'you', label: 'Needs you' },
  { value: 'log', label: 'Log' },
];

const LOG_START = 12;

const LOG_STEP = 25;

const RESULT_TAB = 'result';

const NO_LEAD = 'the board';

const CARD_OWN_CONTROLS = 'button, a, input, textarea, select, summary, [role="menu"], [role="menuitem"], [role="dialog"]';

const FEED_CARD_MIN_WIDTH = 288;

const CONTEXT_LINES = 2;

const CONTEXT_MORE_CHARS = 160;

export { CARD_OWN_CONTROLS, CONTEXT_LINES, CONTEXT_MORE_CHARS, FEED_CARD_MIN_WIDTH, FILTER_OPTIONS, FILTER_STORAGE_KEY, LOG_START, LOG_STEP, NO_LEAD, RESULT_TAB };
