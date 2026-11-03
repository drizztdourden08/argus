/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';
import { buildColumns } from './build-columns';
import { filtersActive } from './filters-active';
import { listedThreads } from './listed-threads';
import { matchesFilters } from './matches-filters';
import { CLOSED_STATUSES } from './model.constants';
import { needsYou } from './needs-you';
import { quietMs } from './quiet-ms';
import type { CardData, CardFacts, KanbanView, ViewInput } from './model.type';

const isClosed = (item: ItemRef): boolean => CLOSED_STATUSES.includes(item.status);

const cardOf = (item: ItemRef, facts: CardFacts, now: number): CardData => {
  const waiting = needsYou(facts.needs);
  return { id: item.id, item, needsYou: waiting, asks: facts.asks, quietMs: quietMs(item, waiting, now), conflicted: facts.conflicted };
};

const conflictedIds = (input: ViewInput): Set<string> =>
  new Set(input.notices.filter((notice) => notice.kind === 'conflict').map((notice) => String(notice.op.target?.id)));

const threadKeeper = (input: ViewInput): ((thread: string) => boolean) => {
  const focus = input.threads.some((thread) => thread.id === input.focus) ? input.focus : '';
  return (thread) => (input.showPaused || !input.paused.has(thread)) && (!focus || thread === focus);
};

const buildView = (input: ViewInput): KanbanView => {
  const { items, threads, needs, asks, filters, prefs, now } = input;
  const conflicted = conflictedIds(input);
  const keeps = threadKeeper(input);
  const live = items.filter((item) => !item.deletedAt && (filters.archived || !item.archivedAt) && keeps(item.thread));
  const cards = live.map((item) => cardOf(item, { needs: needs.get(item.id), asks: asks.get(item.id) ?? [], conflicted: conflicted.has(item.id) }, now));
  const showClosed = prefs.showDone || CLOSED_STATUSES.includes(filters.status);
  const shown = cards.filter((card) => (showClosed || !isClosed(card.item)) && matchesFilters(card.item, filters, card.needsYou, card.conflicted));
  const filtered = filtersActive(filters);
  const seeds = prefs.group === 'thread' && !filtered ? listedThreads(threads) : [];
  return {
    columns: buildColumns({ cards: shown, group: prefs.group, order: prefs.order, showClosed, seeds }),
    cards: shown,
    open: cards.filter((card) => !isClosed(card.item)).length,
    total: cards.length,
    quiet: shown.filter((card) => card.quietMs !== null).length,
    closed: cards.filter((card) => isClosed(card.item)).length,
    filtered,
  };
};

export { buildView };
