/* @layer renderer-app @kind logic */
import { TAB_LOCK_PREFIX } from './hub-transport.constants';
import type { LockManagerLike } from './hub-transport.type';

const holdTabLock = (locks: LockManagerLike, tabId: string): (() => void) => {
  let release: () => void = () => undefined;
  const held = new Promise<void>((resolve) => {
    release = resolve;
  });
  locks.request(`${TAB_LOCK_PREFIX}${tabId}`, {}, () => held).catch(() => undefined);
  return () => release();
};

export { holdTabLock };
