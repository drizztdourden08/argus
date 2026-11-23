/* @layer renderer-app @kind logic */
import type { SortDirection } from '@drizztdourden08/tessera';
import type { FeedFilters } from '../OwnerFeed.type';
import type { AskFeedCard } from './ask.type';
import { cardFacts } from './card-facts';
import { feedKind } from './feed-kind';

const hasText = (query: string, ...parts: (string | null | undefined)[]): boolean => {
  const needle = query.trim().toLowerCase();
  return needle === '' || parts.some((part) => (part ?? '').toLowerCase().includes(needle));
};

const allows = (list: readonly string[], value: string): boolean => list.length === 0 || list.includes(value);

const cardPasses = (filters: FeedFilters) => (card: AskFeedCard): boolean => {
  const facts = cardFacts(card);
  return allows(filters.kinds, feedKind(card)) && allows(filters.authors, facts.thread ?? '') && hasText(filters.query, facts.title, facts.code, facts.thread);
};

const applyFeedFilters = (cards: readonly AskFeedCard[], filters: FeedFilters, direction: SortDirection): AskFeedCard[] => {
  const kept = cards.filter(cardPasses(filters));
  return direction === 'desc' ? kept : [...kept].sort((a, b) => a.at.localeCompare(b.at));
};

export { applyFeedFilters };
