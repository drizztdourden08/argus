/* @layer renderer-app @kind logic */
import { storedListeners } from './stored-listeners';

const watchStored = (listener: () => void): (() => void) => {
  storedListeners.add(listener);
  return () => void storedListeners.delete(listener);
};

export { watchStored };
