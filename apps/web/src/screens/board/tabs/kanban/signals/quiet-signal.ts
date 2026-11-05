/* @layer renderer-app @kind logic */
import { formatAgo } from '../../../../../format';
import { ATTENTION_OF } from '../attention';
import type { CardData } from '../model';
import { MS_PER_HOUR } from '../model/model.constants';
import { OPEN_ITEM } from './signal.constants';
import type { CardSignal } from './signal.type';

const quietSignal = (card: CardData, now: number): CardSignal | null => {
  if (card.quietMs === null) return null;
  const { icon, color } = ATTENTION_OF.quiet;
  const count = Math.floor(card.quietMs / MS_PER_HOUR);
  return { id: 'quiet', icon, count, label: `quiet ${formatAgo(card.item.updatedAt, now)}`, tone: color, opens: OPEN_ITEM };
};

export { quietSignal };
