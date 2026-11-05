/* @layer renderer-app @kind logic */
import type { SortDirection } from '@drizztdourden08/tessera';
import type { BoardWork, FeedCard } from '../../feed-model';
import { cardFacts } from '../behavior/card-facts';
import type { FeedFilters } from '../OwnerFeed.type';
import { feedKind } from './feed-kind';

const hasText = (query: string, ...parts: (string | null | undefined)[]): boolean => {
  const needle = query.trim().toLowerCase();
  return needle === '' || parts.some((part) => (part ?? '').toLowerCase().includes(needle));
};

const allows = (list: readonly string[], value: string): boolean => list.length === 0 || list.includes(value);

const cardPasses = (filters: FeedFilters) => (card: FeedCard): boolean => {
  const facts = cardFacts(card);
  return allows(filters.kinds, feedKind(card)) && allows(filters.authors, facts.thread ?? '') && hasText(filters.query, facts.title, facts.code, facts.thread);
};

const linePasses = (filters: FeedFilters) => (line: BoardWork['lines'][number]): boolean =>
  allows(filters.kinds, 'note') && allows(filters.authors, line.thread) && hasText(filters.query, line.text, line.thread, line.item);

const ordered = <T,>(list: T[], at: (entry: T) => string, direction: SortDirection): T[] =>
  direction === 'desc' ? list : [...list].sort((a, b) => at(a).localeCompare(at(b)));

const applyFeedFilters = (work: BoardWork, filters: FeedFilters, direction: SortDirection): BoardWork => {
  const cards = (list: FeedCard[]) => ordered(list.filter(cardPasses(filters)), (card) => card.at, direction);
  return {
    ...work,
    waiting: cards(work.waiting),
    leads: cards(work.leads),
    notices: cards(work.notices),
    lines: ordered(work.lines.filter(linePasses(filters)), (line) => line.ts, direction),
  };
};

export { applyFeedFilters };
