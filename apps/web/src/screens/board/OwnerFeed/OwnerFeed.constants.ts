/* @layer renderer-app @kind constants */
import type { FeedFilter, FeedFilters } from './OwnerFeed.type';

const FILTER_STORAGE_KEY = 'coord.feed';

const FILTER_OPTIONS: { value: FeedFilter; label: string }[] = [
  { value: 'all', label: 'All' },
  { value: 'you', label: 'Needs you' },
  { value: 'log', label: 'Log' },
];

const FEED_VIEW_LABEL = 'Activity view';

const FEED_KIND_OPTIONS = [
  { value: 'question', label: 'Question' },
  { value: 'decision', label: 'Decision' },
  { value: 'note', label: 'Note' },
  { value: 'test', label: 'Test' },
  { value: 'notice', label: 'Notice' },
];

const FEED_SORT_OPTIONS = [{ id: 'date' as const, label: 'Date' }];

const FEED_VIEW = {
  groups: ['none'] as const,
  orders: ['date'] as const,
  group: 'none' as const,
  order: 'date' as const,
  direction: 'desc' as const,
  filters: { query: '', authors: [], kinds: [] } as FeedFilters,
};

const LOG_START = 12;

const LOG_STEP = 25;

const RESULT_TAB = 'result';

const NO_LEAD = 'the board';


const FEED_CARD_MIN_WIDTH = 288;

const CONTEXT_LINES = 2;

const CONTEXT_MORE_CHARS = 160;

export { CONTEXT_LINES, CONTEXT_MORE_CHARS, FEED_CARD_MIN_WIDTH, FEED_KIND_OPTIONS, FEED_SORT_OPTIONS, FEED_VIEW, FEED_VIEW_LABEL, FILTER_OPTIONS, FILTER_STORAGE_KEY, LOG_START, LOG_STEP, NO_LEAD, RESULT_TAB };
