/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';
import { NO_SIGNALS, attentionOf } from '../attention';
import { buildColumns } from './build-columns';
import { childCounts } from './child-counts';
import { filtersActive } from './filters-active';
import { listedThreads } from './listed-threads';
import { matchesFilters } from './matches-filters';
import { CLOSED_STATUSES, NO_CHILDREN, TRAIL_MIN_DEPTH } from './model.constants';
import { needsYou } from './needs-you';
import { parentTitles } from './parent-titles';
import { quietMs } from './quiet-ms';
import type { CardData, CardFacts, KanbanView, TrailInfo, ViewInput } from './model.type';

const isClosed = (item: ItemRef): boolean => CLOSED_STATUSES.includes(item.status);

const cardOf = (item: ItemRef, facts: CardFacts, now: number): CardData => {
  const waiting = needsYou(facts.needs);
  const quiet = quietMs(item, waiting, now);
  const { asks, signals, conflicted, needs, children, trail } = facts;
  const attention = attentionOf({ item, needs, asks, signals, quietMs: quiet });
  return { id: item.id, item, needsYou: waiting, needs, children, trail, asks, signals, attention, quietMs: quiet, conflicted };
};

const longTrail = (trail: TrailInfo | undefined): TrailInfo | null => (trail && trail.depth >= TRAIL_MIN_DEPTH ? trail : null);

const conflictedIds = (input: ViewInput): Set<string> =>
  new Set(input.notices.filter((notice) => notice.kind === 'conflict').map((notice) => String(notice.op.target?.id)));

const threadKeeper = (input: ViewInput): ((thread: string) => boolean) => {
  const focus = input.threads.some((thread) => thread.id === input.focus) ? input.focus : '';
  return (thread) => (input.showPaused || !input.paused.has(thread)) && (!focus || thread === focus);
};

const buildView = (input: ViewInput): KanbanView => {
  const { items, threads, needs, asks, signals, trails, filters, prefs, now } = input;
  const conflicted = conflictedIds(input);
  const keeps = threadKeeper(input);
  const live = items.filter((item) => !item.deletedAt && (filters.archived || !item.archivedAt) && keeps(item.thread));
  const children = childCounts(items);
  const facts = (item: ItemRef): CardFacts => ({
    needs: needs.get(item.id),
    asks: asks.get(item.id) ?? [],
    signals: signals.get(item.id) ?? NO_SIGNALS,
    conflicted: conflicted.has(item.id),
    children: children.get(item.id) ?? NO_CHILDREN,
    trail: longTrail(trails.get(item.id)),
  });
  const cards = live.map((item) => cardOf(item, facts(item), now));
  const showClosed = prefs.showDone || CLOSED_STATUSES.includes(filters.status);
  const shown = cards.filter((card) => (showClosed || !isClosed(card.item)) && matchesFilters(card.item, filters, card.needsYou, card.conflicted));
  const filtered = filtersActive(filters);
  const seeds = prefs.group === 'thread' && !filtered ? listedThreads(threads) : [];
  return {
    columns: buildColumns({ cards: shown, group: prefs.group, order: prefs.order, showClosed, seeds, parents: parentTitles(items) }),
    cards: shown,
    open: cards.filter((card) => !isClosed(card.item)).length,
    total: cards.length,
    quiet: shown.filter((card) => card.quietMs !== null).length,
    closed: cards.filter((card) => isClosed(card.item)).length,
    filtered,
  };
};

export { buildView };
