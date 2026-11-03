/* @layer renderer-app @kind test */
import type { LockManagerLike } from '../../src/data/hub/hub-transport.type';

const lateLocks = (locks: LockManagerLike) => {
  const pending: Array<() => void> = [];
  const request: LockManagerLike['request'] = (name, options, callback) =>
    new Promise((resolve) => {
      pending.push(() => resolve(locks.request(name, options, callback)));
    });
  const arrive = (): void => {
    for (const run of pending.splice(0)) run();
  };
  return { request, arrive };
};

export { lateLocks };
