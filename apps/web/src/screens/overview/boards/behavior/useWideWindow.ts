/* @layer renderer-app @kind logic */
import { useSyncExternalStore } from 'react';
import { WIDE_QUERY } from '../boards-view.constants';

const watch = (listener: () => void): (() => void) => {
  if (typeof window.matchMedia !== 'function') return () => undefined;
  const query = window.matchMedia(WIDE_QUERY);
  query.addEventListener('change', listener);
  return () => query.removeEventListener('change', listener);
};

const wideNow = (): boolean => typeof window.matchMedia === 'function' && window.matchMedia(WIDE_QUERY).matches;

const useWideWindow = (): boolean => useSyncExternalStore(watch, wideNow, () => false);

export { useWideWindow };
