/* @layer renderer-app @kind logic */
import type { Slot } from './slot-store.type';

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

export { createSlot };
