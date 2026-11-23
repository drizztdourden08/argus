/* @layer renderer-app @kind logic */
import type { Option } from '../../shared';
import type { AskFeedCard } from './ask.type';
import { cardFacts } from './card-facts';

const feedAuthors = (cards: readonly AskFeedCard[]): Option[] => {
  const names = new Set(cards.map((card) => cardFacts(card).thread ?? ''));
  return [...names].filter((name) => name !== '').sort().map((name) => ({ value: name, label: name }));
};

export { feedAuthors };
