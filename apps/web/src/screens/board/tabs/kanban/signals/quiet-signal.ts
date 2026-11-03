/* @layer renderer-app @kind logic */
import { formatAgo } from '../../../../../format';
import { ATTENTION_OF } from '../attention';
import type { CardData } from '../model';
import { OPEN_ITEM } from './signal.constants';
import type { CardSignal } from './signal.type';

const quietSignal = (card: CardData, now: number): CardSignal | null => {
  if (card.quietMs === null) return null;
  const { icon, color } = ATTENTION_OF.quiet;
  return { id: 'quiet', icon, label: `quiet ${formatAgo(card.item.updatedAt, now)}`, tone: color, opens: OPEN_ITEM };
};

export { quietSignal };
