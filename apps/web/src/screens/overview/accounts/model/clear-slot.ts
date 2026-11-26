/* @layer renderer-app @kind logic */
import type { SlotPair, SlotSide } from '../next.type';

const clearSlot = (pair: SlotPair, side: SlotSide): SlotPair => ({ ...pair, [side]: null });

export { clearSlot };
