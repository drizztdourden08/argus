/* @layer renderer-app @kind logic */
import { useSyncExternalStore } from 'react';

interface Slot {
  set: (node: HTMLElement | null) => void;
  subscribe: (listener: () => void) => () => void;
  get: () => HTMLElement | null;
}

const createSlot = (): Slot => {
  let node: HTMLElement | null = null;
  const listeners = new Set<() => void>();
  return {
    set: (next) => {
      node = next;
      listeners.forEach((listener) => listener());
    },
    subscribe: (listener) => {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    get: () => node,
  };
};

const useSlotNode = (slot: Slot): HTMLElement | null => useSyncExternalStore(slot.subscribe, slot.get);

const toolsSlot = createSlot();

const filtersSlot = createSlot();

export { filtersSlot, toolsSlot, useSlotNode };
