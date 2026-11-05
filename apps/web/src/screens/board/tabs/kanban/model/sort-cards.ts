/* @layer renderer-app @kind logic */
import type { SortDirection } from '@drizztdourden08/tessera';
import type { CardData, ItemOrder } from './model.type';
import { isNatural } from './is-natural';

const byRank = (a: CardData, b: CardData): number =>
  (a.item.rank ?? 0) - (b.item.rank ?? 0) || String(a.item.createdAt).localeCompare(String(b.item.createdAt)) || a.id.localeCompare(b.id);

const mostRecent = (a: CardData, b: CardData): number => String(b.item.updatedAt).localeCompare(String(a.item.updatedAt));

const sortCards = (cards: readonly CardData[], order: ItemOrder, direction: SortDirection | null): CardData[] => {
  const sorted = [...cards].sort(order === 'rank' ? byRank : mostRecent);
  return isNatural(order, direction) ? sorted : sorted.reverse();
};

export { sortCards };
