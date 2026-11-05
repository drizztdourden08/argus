/* @layer renderer-app @kind logic */
import { itemTypeOf } from '../../../../../data/registry';
import { acProgress } from './ac-progress';
import { childrenProgress } from './children-progress';
import { runProgress } from './run-progress';
import { typeOf } from './type-of';
import type { CardData, CardProgress } from './model.type';

const present = (parts: readonly (CardProgress | null)[]): CardProgress[] => parts.filter((part): part is CardProgress => part !== null);

const withChildrenWord = (part: CardProgress | null): CardProgress | null => (part ? { ...part, text: `${part.text} children` } : null);

const combined = (card: CardData): CardProgress[] => {
  const [children, ac] = [childrenProgress(card.children), acProgress(card.item)];
  return children && ac ? [children, { ...ac, text: `+ AC ${ac.text}` }] : present([children, ac]);
};

const cardProgress = (card: CardData): CardProgress[] => {
  switch (itemTypeOf(typeOf(card.item))?.card.progress) {
    case 'ac':
      return present([acProgress(card.item)]);
    case 'children':
      return present([withChildrenWord(childrenProgress(card.children))]);
    case 'children-ac':
      return combined(card);
    case 'run':
      return present([runProgress(card.item, card.lastRun)]);
    case 'checks':
    case 'none':
    case undefined:
      return [];
  }
};

export { cardProgress };
