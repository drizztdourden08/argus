/* @layer renderer-app @kind logic */
import type { WorkItem } from '../../../data/types';
import { buildFeed } from './build-feed';
import { FINISHED_ITEM_STATUSES, OP_KINDS } from './feed.constants';
import type { BoardWork, FeedCard, WorkSources } from './feed.type';
import { itemNeeds } from './item-needs';
import { ownerMarks } from './owner-marks';

const isVisible = (doc: { archivedAt?: string; deletedAt?: string }): boolean => !doc.archivedAt && !doc.deletedAt;

const byRank = (a: WorkItem, b: WorkItem): number => (a.rank ?? 0) - (b.rank ?? 0) || a.id.localeCompare(b.id, undefined, { numeric: true });

const openItems = (items: WorkItem[]): WorkItem[] =>
  items.filter((item) => isVisible(item) && !FINISHED_ITEM_STATUSES.includes(item.status)).sort(byRank);

const indexOf = (items: WorkItem[]): ReadonlyMap<string, WorkItem> => new Map(items.filter(isVisible).map((item) => [item.id, item]));

const isAsk = (card: FeedCard): boolean => card.type === 'decision' || card.type === 'question' || card.type === 'message';

const buildWork = (sources: WorkSources): Omit<BoardWork, 'ready'> => {
  const entries = sources.entries.filter((entry) => isVisible(entry) && !OP_KINDS.includes(entry.kind));
  const actions = sources.actions.filter(isVisible);
  const cards = buildFeed({ entries, actions, tests: sources.tests, items: sources.items.filter(isVisible), marks: ownerMarks(entries) });
  return {
    asks: cards.filter(isAsk),
    items: openItems(sources.items),
    index: indexOf(sources.items),
    needs: itemNeeds(cards, entries),
  };
};

export { buildWork };
