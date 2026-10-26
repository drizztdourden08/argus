/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';
import { buildColumns } from './build-columns';
import { filtersActive } from './filters-active';
import { listedThreads } from './listed-threads';
import { matchesFilters } from './matches-filters';
import { CLOSED_STATUSES } from './model.constants';
import { needsYou } from './needs-you';
import { quietMs } from './quiet-ms';
import type { CardData, KanbanView, Needs, ViewInput } from './model.type';

const isClosed = (item: ItemRef): boolean => CLOSED_STATUSES.includes(item.status);

const cardOf = (item: ItemRef, needs: Needs | undefined, conflicted: boolean, now: number): CardData => {
  const waiting = needsYou(needs);
  return { id: item.id, item, needsYou: waiting, decisions: needs?.decisions ?? 0, quietMs: quietMs(item, waiting, now), conflicted };
};

const conflictedIds = (input: ViewInput): Set<string> =>
  new Set(input.notices.filter((notice) => notice.kind === 'conflict').map((notice) => String(notice.op.target?.id)));

const buildView = (input: ViewInput): KanbanView => {
  const { items, threads, needs, filters, prefs, now } = input;
  const conflicted = conflictedIds(input);
  const live = items.filter((item) => !item.deletedAt && (filters.archived || !item.archivedAt));
  const cards = live.map((item) => cardOf(item, needs.get(item.id), conflicted.has(item.id), now));
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
