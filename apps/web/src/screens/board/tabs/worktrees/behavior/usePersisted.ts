/* @layer renderer-app @kind logic */
import { useCallback, useState } from 'react';

const readChoice = (key: string): string | null => {
  try {
    return window.localStorage.getItem(key);
  } catch {
    return null;
  }
};

const writeChoice = (key: string, value: string): boolean => {
  try {
    window.localStorage.setItem(key, value);
    return true;
  } catch {
    return false;
  }
};

const usePersisted = <T extends string>(key: string, allowed: readonly T[], fallback: T): [T, (next: T) => void] => {
  const [value, setValue] = useState<T>(() => allowed.find((option) => option === readChoice(key)) ?? fallback);
  const choose = useCallback((next: T) => {
    setValue(next);
    writeChoice(key, next);
  }, [key]);
  return [value, choose];
};

export { usePersisted };
