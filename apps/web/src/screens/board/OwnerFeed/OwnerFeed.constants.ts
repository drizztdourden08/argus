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

const CONTEXT_LINES = 2;

const CONTEXT_MORE_CHARS = 160;

export { CONTEXT_LINES, CONTEXT_MORE_CHARS, FILTER_OPTIONS, FILTER_STORAGE_KEY, LOG_START, LOG_STEP, RESULT_TAB };
