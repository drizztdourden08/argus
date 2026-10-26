/* @layer renderer-app @kind logic */
import type { Card } from '../../../../data/types';
import { hotOf } from './hot-of';

const sortCards = (cards: readonly Card[]): Card[] =>
  [...cards].sort((a, b) => Number(hotOf(b) > 0) - Number(hotOf(a) > 0) || a.name.localeCompare(b.name));

export { sortCards };
