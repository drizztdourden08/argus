/* @layer renderer-app @kind logic */
import type { CardData } from './model.type';

const clusterGroups = (cards: readonly CardData[]): CardData[] => {
  const seen = new Set<string>();
  return cards.flatMap((card) => {
    if (!card.group) return [card];
    if (seen.has(card.group)) return [];
    seen.add(card.group);
    return cards.filter((other) => other.group === card.group);
  });
};

export { clusterGroups };
