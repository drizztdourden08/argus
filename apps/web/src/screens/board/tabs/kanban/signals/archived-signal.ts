/* @layer renderer-app @kind logic */
import type { CardData } from '../model';
import { OPEN_ITEM } from './signal.constants';
import type { CardSignal } from './signal.type';

const archivedSignal = (card: CardData): CardSignal | null =>
  card.item.archivedAt ? { id: 'archived', icon: 'archive', label: 'archived', tone: 'neutral', opens: OPEN_ITEM } : null;

export { archivedSignal };
