/* @layer renderer-app @kind logic */
import { itemTypeOf } from '../../../../../data/registry';
import { childSegments } from './child-segments';
import { depSegments } from './dep-segments';
import { WITH_CHILDREN, WITH_CRITERIA } from './model.constants';
import type { CardBars, CardData } from './model.type';
import { typeOf } from './type-of';

const NONE: CardBars = { work: [], criteria: [] };

const cardBars = (card: CardData): CardBars => {
  const progress = itemTypeOf(typeOf(card.item))?.card.progress;
  if (progress === undefined || progress === 'run') return NONE;
  const children = WITH_CHILDREN.includes(progress) ? childSegments(card.children) : [];
  return { work: [...children, ...depSegments(card.item)], criteria: WITH_CRITERIA.includes(progress) ? (card.item.ac ?? []) : [] };
};

export { cardBars };
