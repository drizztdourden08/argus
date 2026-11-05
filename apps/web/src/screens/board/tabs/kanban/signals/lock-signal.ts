/* @layer renderer-app @kind logic */
import { ATTENTION_OF, lockCauses } from '../attention';
import type { CardData } from '../model';
import { causeText } from './cause-text';
import { OPEN_ITEM } from './signal.constants';
import type { CardSignal } from './signal.type';
import { whoAvatar } from './who-avatar';

const lockSignal = (card: CardData): CardSignal | null => {
  const causes = lockCauses(card.item, card.asks);
  if (causes.length === 0) return null;
  const { icon, color } = ATTENTION_OF.blocked;
  const who = causes.find((cause) => cause.who)?.who;
  const signal: CardSignal = { id: 'blocked', icon, count: causes.length, label: `Blocked by ${causes.map(causeText).join('; ')}`, tone: color, opens: OPEN_ITEM };
  return who ? { ...signal, avatar: whoAvatar(who) } : signal;
};

export { lockSignal };
