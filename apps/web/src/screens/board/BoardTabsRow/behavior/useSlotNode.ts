/* @layer renderer-app @kind logic */
import { useSyncExternalStore } from 'react';
import type { Slot } from './slot-store.type';

const useSlotNode = (slot: Slot): HTMLElement | null => useSyncExternalStore(slot.subscribe, slot.get);

export { useSlotNode };
