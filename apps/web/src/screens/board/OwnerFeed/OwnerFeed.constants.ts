/* @layer renderer-app @kind constants */
import type { FeedFilters } from './OwnerFeed.type';

const FEED_KIND_OPTIONS = [
  { value: 'question', label: 'Question' },
  { value: 'decision', label: 'Decision' },
  { value: 'note', label: 'Note' },
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

const CONTEXT_LINES = 2;

const CONTEXT_MORE_CHARS = 160;

export { CONTEXT_LINES, CONTEXT_MORE_CHARS, FEED_KIND_OPTIONS, FEED_SORT_OPTIONS, FEED_VIEW };
