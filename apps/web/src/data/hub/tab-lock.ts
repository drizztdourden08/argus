/* @layer renderer-app @kind logic */
import { TAB_LOCK_PREFIX } from './hub-transport.constants';
import type { LockManagerLike } from './hub-transport.type';

const holdTabLock = (locks: LockManagerLike | null, tabId: string, onHeld: () => void): (() => void) => {
  if (!locks) {
    onHeld();
    return () => undefined;
  }
  let release: () => void = () => undefined;
  const held = new Promise<void>((resolve) => {
    release = resolve;
  });
  const hold = (): Promise<void> => {
    onHeld();
    return held;
  };
  locks.request(`${TAB_LOCK_PREFIX}${tabId}`, {}, hold).catch(() => onHeld());
  return () => release();
};

export { holdTabLock };
