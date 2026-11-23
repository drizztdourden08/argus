/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';
import { itemTypeOf } from '../../../../../data/registry';
import { NO_SIGNALS, attentionOf } from '../attention';
import { buildColumns } from './build-columns';
import { limitsOf } from './column-limits';
import { targetsOf } from './column-targets';
import { columnsOf } from './config-columns';
import { childCounts } from './child-counts';
import { groupOf } from './group-of';
import { filtersActive } from './filters-active';
import { latestRuns } from './latest-runs';
import { listedThreads } from './listed-threads';
import { matchesFilters } from './matches-filters';
import { CLOSED_STATUSES, NO_CHILDREN, TRAIL_MIN_DEPTH } from './model.constants';
import { needsYou } from './needs-you';
import { parentItems } from './parent-items';
import { quietMs } from './quiet-ms';
import { quietLimits } from './quiet-limits';
import { typeOf } from './type-of';
import type { CardData, CardFacts, ItemGroup, KanbanView, TrailInfo, ViewInput } from './model.type';

const onKanban = (item: ItemRef): boolean => !itemTypeOf(typeOf(item))?.hideOnKanban;

const isClosed = (item: ItemRef): boolean => CLOSED_STATUSES.includes(item.status);

const cardOf = (item: ItemRef, facts: CardFacts, now: number, limits: Readonly<Record<string, number>>): CardData => {
  const waiting = needsYou(facts.needs);
  const quiet = quietMs(item, waiting, now, limits);
  const { asks, signals, conflicted, needs, children, lastRun, trail } = facts;
  const attention = attentionOf({ item, needs, asks, signals, quietMs: quiet });
  return { id: item.id, item, needsYou: waiting, needs, children, lastRun, trail, asks, signals, attention, quietMs: quiet, conflicted };
};

const longTrail = (trail: TrailInfo | undefined): TrailInfo | null => (trail && trail.depth >= TRAIL_MIN_DEPTH ? trail : null);

const conflictedIds = (input: ViewInput): Set<string> =>
  new Set(input.notices.filter((notice) => notice.kind === 'conflict').map((notice) => String(notice.op.target?.id)));

const threadKeeper = (input: ViewInput): ((thread: string) => boolean) => {
  const focus = input.threads.some((thread) => thread.id === input.focus) ? input.focus : '';
  return (thread) => (input.showPaused || !input.paused.has(thread)) && (!focus || thread === focus);
};

const grouped = (cards: CardData[], group: ItemGroup): CardData[] => {
  const byId = new Map(cards.map((card) => [card.id, card.item]));
  return cards.map((card) => {
    const container = groupOf(card.item, group, byId);
    return container ? { ...card, group: container } : card;
  });
};

const buildView = (input: ViewInput): KanbanView => {
  const { items, threads, needs, asks, signals, trails, filters, prefs, now } = input;
  const columns = columnsOf(input.config);
  const configured = prefs.group === 'status' ? columns : null;
  const limits = quietLimits(input.config, columns);
  const conflicted = conflictedIds(input);
  const keeps = threadKeeper(input);
  const live = items.filter((item) => onKanban(item) && !item.deletedAt && (filters.archived || !item.archivedAt) && keeps(item.thread));
  const children = childCounts(items);
  const runs = latestRuns(items);
  const facts = (item: ItemRef): CardFacts => ({
    needs: needs.get(item.id),
    asks: asks.get(item.id) ?? [],
    signals: signals.get(item.id) ?? NO_SIGNALS,
    conflicted: conflicted.has(item.id),
    children: children.get(item.id) ?? NO_CHILDREN,
    lastRun: runs.get(item.id) ?? null,
    trail: longTrail(trails.get(item.id)),
  });
  const parents = parentItems(items);
  const cards = live.map((item) => cardOf(item, facts(item), now, limits));
  const showClosed = prefs.showDone || CLOSED_STATUSES.includes(filters.status);
  const visible = cards.filter((card) => (showClosed || !isClosed(card.item)) && matchesFilters(card.item, filters, card.needsYou, card.conflicted));
  const shown = grouped(visible, prefs.group);
  const containers = new Set(shown.flatMap((card) => (card.group ? [card.group] : [])));
  const filtered = filtersActive(filters);
  const seeds = prefs.group === 'thread' && !filtered ? listedThreads(threads) : [];
  return {
    columns: buildColumns({ cards: shown.filter((card) => !containers.has(card.id)), group: prefs.group, order: prefs.order, direction: prefs.direction, showClosed, seeds, parents, config: input.config }),
    cards: shown,
    open: cards.filter((card) => !isClosed(card.item)).length,
    total: cards.length,
    quiet: shown.filter((card) => card.quietMs !== null).length,
    closed: cards.filter((card) => isClosed(card.item)).length,
    filtered,
    parents,
    limits: limitsOf(configured),
    targets: targetsOf(configured),
  };
};

export { buildView };
