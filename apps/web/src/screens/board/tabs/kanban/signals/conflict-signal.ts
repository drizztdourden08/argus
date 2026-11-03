/* @layer renderer-app @kind logic */
import type { CardData } from '../model';
import { OPEN_ITEM } from './signal.constants';
import type { CardSignal } from './signal.type';

const conflictSignal = (card: CardData): CardSignal | null =>
  card.conflicted ? { id: 'conflict', icon: 'triangle-alert', label: 'conflict', tone: 'danger', opens: OPEN_ITEM } : null;

export { conflictSignal };
