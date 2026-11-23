/* @layer renderer-app @kind logic */
import type { CardData } from './model.type';

const withTasks = (members: readonly CardData[]): CardData[] => {
  const ids = new Set(members.map((card) => card.id));
  const place = (card: CardData): CardData[] => [card, ...members.filter((other) => other.item.parent === card.id).flatMap(place)];
  return members.filter((card) => !card.item.parent || !ids.has(card.item.parent)).flatMap(place);
};

const clusterGroups = (cards: readonly CardData[]): CardData[] => {
  const keys = new Set(cards.flatMap((card) => (card.group ? [card.group] : [])));
  const seen = new Set<string>();
  return cards.flatMap((card) => {
    if (keys.has(card.id)) return [];
    if (!card.group) return [card];
    if (seen.has(card.group)) return [];
    seen.add(card.group);
    return withTasks(cards.filter((other) => other.group === card.group));
  });
};

export { clusterGroups };
