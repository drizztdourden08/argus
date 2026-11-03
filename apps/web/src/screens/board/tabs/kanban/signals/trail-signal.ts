/* @layer renderer-app @kind logic */
import type { CardData } from '../model';
import { OPEN_ITEM, TRAIL_SIGNAL } from './signal.constants';
import type { CardSignal } from './signal.type';

const trailSignal = (card: CardData): CardSignal | null => {
  const { trail } = card;
  if (!trail) return null;
  return { id: TRAIL_SIGNAL, icon: 'signpost', count: trail.depth, label: `trail ${trail.depth}, ${trail.open} open`, tone: 'neutral', opens: OPEN_ITEM };
};

export { trailSignal };
