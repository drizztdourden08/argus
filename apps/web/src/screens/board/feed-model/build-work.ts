/* @layer renderer-app @kind logic */
import type { WorkItem } from '../../../data/types';
import { buildFeed } from './build-feed';
import { FINISHED_ITEM_STATUSES, OP_KINDS } from './feed.constants';
import type { BoardWork, WorkSources } from './feed.type';
import { itemNeeds } from './item-needs';
import { logLines } from './log-lines';
import { ownerMarks } from './owner-marks';
import { sectionOf } from './section-of';

const isVisible = (doc: { archivedAt?: string; deletedAt?: string }): boolean => !doc.archivedAt && !doc.deletedAt;

const byRank = (a: WorkItem, b: WorkItem): number => (a.rank ?? 0) - (b.rank ?? 0) || a.id.localeCompare(b.id, undefined, { numeric: true });

const openItems = (items: WorkItem[]): WorkItem[] =>
  items.filter((item) => isVisible(item) && !FINISHED_ITEM_STATUSES.includes(item.status)).sort(byRank);

const indexOf = (items: WorkItem[]): ReadonlyMap<string, WorkItem> => new Map(items.filter(isVisible).map((item) => [item.id, item]));

const buildWork = (sources: WorkSources): Omit<BoardWork, 'ready'> => {
  const entries = sources.entries.filter((entry) => isVisible(entry) && !OP_KINDS.includes(entry.kind));
  const closed = new Set(sources.builds.filter((build) => build.closedAt).map((build) => build.id));
  const actions = sources.actions.filter(isVisible);
  const cards = buildFeed({ entries, actions, tests: sources.tests, closed, marks: ownerMarks(entries) });
  return {
    waiting: cards.filter((card) => sectionOf(card) === 'waiting'),
    leads: cards.filter((card) => sectionOf(card) === 'leads'),
    notices: cards.filter((card) => sectionOf(card) === 'log'),
    lines: logLines(sources.log),
    items: openItems(sources.items),
    index: indexOf(sources.items),
    needs: itemNeeds(cards, entries),
  };
};

export { buildWork };
