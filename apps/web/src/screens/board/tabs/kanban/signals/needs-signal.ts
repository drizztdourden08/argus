/* @layer renderer-app @kind logic */
import { ATTENTION_OF } from '../attention';
import type { CardData } from '../model';
import { NEEDS_SIGNAL, OPEN_ITEM } from './signal.constants';
import type { CardSignal } from './signal.type';

const needsSignal = (card: CardData): CardSignal | null => {
  if (!card.needsYou) return null;
  const lead = card.attention.find((kind) => kind === 'question' || kind === 'todo') ?? 'test-pending';
  return { id: NEEDS_SIGNAL, icon: ATTENTION_OF[lead].icon, label: 'needs you', tone: 'danger', opens: OPEN_ITEM };
};

export { needsSignal };
