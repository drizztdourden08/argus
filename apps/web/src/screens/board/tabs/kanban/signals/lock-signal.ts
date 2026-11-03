/* @layer renderer-app @kind logic */
import { ATTENTION_OF, lockCauses } from '../attention';
import type { CardData } from '../model';
import { causeText } from './cause-text';
import { OPEN_ITEM } from './signal.constants';
import type { CardSignal } from './signal.type';

const lockSignal = (card: CardData): CardSignal | null => {
  const causes = lockCauses(card.item, card.asks);
  if (causes.length === 0) return null;
  const { icon, color } = ATTENTION_OF.blocked;
  return { id: 'blocked', icon, count: causes.length, label: `Blocked by ${causes.map(causeText).join('; ')}`, tone: color, opens: OPEN_ITEM };
};

export { lockSignal };
