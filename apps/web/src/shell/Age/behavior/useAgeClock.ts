/* @layer renderer-app @kind logic */
import { useSyncExternalStore } from 'react';
import { AGE_TICK_MS } from '../Age.constants';

const listeners = new Set<() => void>();
let snapshot = Date.now();
let timer: ReturnType<typeof setInterval> | undefined;

const tick = (): void => {
  snapshot = Date.now();
  listeners.forEach((listener) => listener());
};

const subscribe = (listener: () => void): (() => void) => {
  if (listeners.size === 0) {
    snapshot = Date.now();
    timer = setInterval(tick, AGE_TICK_MS);
  }
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
    if (listeners.size === 0) clearInterval(timer);
  };
};

const read = (): number => snapshot;

const useAgeClock = (): number => useSyncExternalStore(subscribe, read, read);

export { useAgeClock };
