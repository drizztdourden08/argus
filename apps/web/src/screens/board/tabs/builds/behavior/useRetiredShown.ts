/* @layer renderer-app @kind logic */
import { useCallback, useState } from 'react';
import { RETIRED_KEY } from '../builds.constants';
import type { RetiredFlag } from '../builds.type';

const readFlag = (): boolean => {
  try {
    return localStorage.getItem(RETIRED_KEY) === '1';
  } catch {
    return false;
  }
};

const writeFlag = (open: boolean): boolean => {
  try {
    localStorage.setItem(RETIRED_KEY, open ? '1' : '0');
    return true;
  } catch {
    return false;
  }
};

const useRetiredShown = (): RetiredFlag => {
  const [shown, setShown] = useState(readFlag);
  const set = useCallback((open: boolean) => {
    setShown(open);
    writeFlag(open);
  }, []);
  return { shown, set };
};

export { useRetiredShown };
