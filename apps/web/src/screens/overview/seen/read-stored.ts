/* @layer renderer-app @kind logic */
import { toMillis } from '../../../format';

const readStored = (key: string): number => {
  try {
    return toMillis(localStorage.getItem(key));
  } catch {
    return Number.NaN;
  }
};

export { readStored };
