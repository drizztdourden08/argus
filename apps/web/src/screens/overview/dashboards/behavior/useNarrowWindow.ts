/* @layer renderer-app @kind logic */
import { useSyncExternalStore } from 'react';
import { NARROW_QUERY } from '../dashboards.constants';

const watch = (listener: () => void): (() => void) => {
  if (typeof window.matchMedia !== 'function') return () => undefined;
  const query = window.matchMedia(NARROW_QUERY);
  query.addEventListener('change', listener);
  return () => query.removeEventListener('change', listener);
};

const narrowNow = (): boolean => typeof window.matchMedia === 'function' && window.matchMedia(NARROW_QUERY).matches;

const useNarrowWindow = (): boolean => useSyncExternalStore(watch, narrowNow, () => false);

export { useNarrowWindow };
