/* @layer renderer-app @kind logic */
import { notifyStored } from './notify-stored';

const writeStored = (key: string, value: string): boolean => {
  try {
    localStorage.setItem(key, value);
  } catch {
    return false;
  }
  notifyStored();
  return true;
};

export { writeStored };
