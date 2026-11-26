/* @layer renderer-app @kind logic */
import type { SlotPair, SlotSide } from '../next.type';

const placeSlot = (pair: SlotPair, side: SlotSide, key: string): SlotPair => {
  if (side === 'left') return { left: key, right: pair.right === key ? pair.left : pair.right };
  return { left: pair.left === key ? pair.right : pair.left, right: key };
};

export { placeSlot };
