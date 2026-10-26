/* @layer renderer-app @kind logic */
import { storedListeners } from './stored-listeners';

const notifyStored = (): void => {
  for (const listener of storedListeners) listener();
};

export { notifyStored };
