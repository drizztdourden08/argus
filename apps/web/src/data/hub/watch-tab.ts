/* @layer renderer-app @kind logic */
import { TAB_LOCK_PREFIX } from './hub-transport.constants';
import type { LockManagerLike } from './hub-transport.type';

const watchTab = (locks: LockManagerLike, tabId: string, onGone: () => void): (() => void) => {
  const abort = new AbortController();
  locks.request(`${TAB_LOCK_PREFIX}${tabId}`, { signal: abort.signal }, onGone).catch(() => undefined);
  return () => abort.abort();
};

export { watchTab };
