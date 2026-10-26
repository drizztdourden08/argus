/* @layer renderer-app @kind logic */
import type { CardData, ItemOrder } from './model.type';

const byRank = (a: CardData, b: CardData): number =>
  (a.item.rank ?? 0) - (b.item.rank ?? 0) || String(a.item.createdAt).localeCompare(String(b.item.createdAt)) || a.id.localeCompare(b.id);

const mostRecent = (a: CardData, b: CardData): number => String(b.item.updatedAt).localeCompare(String(a.item.updatedAt));

const sortCards = (cards: readonly CardData[], order: ItemOrder): CardData[] => [...cards].sort(order === 'rank' ? byRank : mostRecent);

export { sortCards };
